use super::{
    block::{ Block, BlockConfigurer},
    chain::{Blockchain, BlockchainOperation},
    mempool::{MemPool, MemPoolOperation},
    metadata::{ChainMetaData, ChainMetaDataOperation},
    transaction::Transaction,
};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use sha3::{Digest, Sha3_256};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{sync::RwLock, time::sleep};

#[derive(Debug, Clone)]
pub struct Node {
    pub client_tx_sender: Sender<Transaction>,
    pub client_tx_receiver: Receiver<Transaction>,

    pub proposed_block_sender: Sender<Block>,
    pub proposed_block_receiver: Receiver<Block>,

    pub mined_block_sender: Sender<Block>,
    pub mined_block_receiver: Receiver<Block>,

    pub mempool: Arc<RwLock<MemPool>>,
    pub chain: Arc<RwLock<Blockchain>>,
}

impl Default for Node {
    fn default() -> Self {
        let (client_tx_sender, client_tx_receiver) = async_channel::unbounded();
        let (proposed_block_sender, proposed_block_receiver) = async_channel::unbounded();
        let (mined_block_sender, mined_block_receiver) = async_channel::unbounded();
        Self {
            client_tx_sender,
            client_tx_receiver,

            proposed_block_sender,
            proposed_block_receiver,

            mined_block_sender,
            mined_block_receiver,

            mempool: Arc::new(RwLock::new(MemPool::default())),
            chain: Arc::new(RwLock::new(Blockchain::default())),
        }
    }
}

impl Node {
    fn verify_block_hash(hash: String, difficulty: usize) -> bool {
        let hash_binding = hash.as_str();
        &hash_binding[0..difficulty] == "0".repeat(difficulty)
    }

    fn calculate_block_hash(block: Block) -> String {
        let mut hasher = Sha3_256::new();

        let hash_str = format!("{}{}{}{}", block.timestamp(), block.tx_count(), block.nonce(), block.prev_hash());
        hasher.update(hash_str);

        for tx in block.transactions() {
            let hash_str = format!("{:?}", tx);
            hasher.update(hash_str);
        }

        let hash = format!("{:x}", hasher.finalize());
        hash
    }
}

#[async_trait]
pub trait TxProcesser {
    async fn add_tx_to_pool(&self);
    async fn run_tx_receiver(&self) -> Result<(), String>;
}

#[async_trait]
impl TxProcesser for Node {
    async fn add_tx_to_pool(&self) {
        loop {
            if let Ok(tx) = self.client_tx_receiver.recv().await {
                let mut proc_mempool = self.mempool.write().await;
                let _ = proc_mempool.add_transaction(tx.clone()).await;
            }
        }
    }

    async fn run_tx_receiver(&self) -> Result<(), String> {
        let node = self.clone();
        tokio::spawn(async move {
            node.add_tx_to_pool().await;
        });

        Ok(())
    }
}

#[async_trait]
pub trait Proposer {
    async fn build_block(&self) -> Result<Block, String>;
    async fn send_propose_block(&self, block: Block) -> Result<(), String>;
    async fn propose_new_block(&self);
    async fn run_proposer(&self) -> Result<(), String>;
}

#[async_trait]
impl Proposer for Node {
    async fn build_block(&self) -> Result<Block, String> {
        let (block_tx_pickup_period, block_size) = {
            let chain_metadata = ChainMetaData::default();
            (
                chain_metadata.get_block_tx_pickup_period().unwrap(),
                chain_metadata.get_block_size().unwrap(),
            )
        };

        let mut block = Block::default();

        let time_limit = SystemTime::now() + Duration::from_millis(block_tx_pickup_period as u64);

        for _ in 0..block_size {
            let mut proc_mempool = self.mempool.write().await;
            match proc_mempool.pickup_transaction().await {
                Ok(tx) => {
                    block.add_transaction(tx);
                }
                Err(_) => {}
            };

            if SystemTime::now() > time_limit {
                break;
            }
        }

        let proc_chain = self.chain.write().await;
        let prev_hash = proc_chain.get_leaf().unwrap();
        block.set_prev_hash(prev_hash);

        Ok(block)
    }

    async fn send_propose_block(&self, block: Block) -> Result<(), String> {
        self.proposed_block_sender.send(block).await.unwrap();
        Ok(())
    }

    async fn propose_new_block(&self) {
        let (block_gen_slot, block_gen_period) = {
            let chain_metadata = ChainMetaData::default();
            (
                chain_metadata.get_block_gen_slot().unwrap(),
                chain_metadata.get_block_gen_period().unwrap(),
            )
        };
        loop {
            sleep(Duration::from_millis(block_gen_slot as u64)).await;
            // let block = self.build_block().await;

            let block_builder = self.build_block();

            match tokio::time::timeout(
                Duration::from_millis(block_gen_period as u64),
                block_builder,
            )
            .await
            {
                Ok(Ok(block)) => {
                    self.send_propose_block(block).await.unwrap();
                }
                Ok(Err(e)) => {
                    println!("Failed proposing a new block:\n{:?}", e.to_string());
                }
                Err(_) => {
                    println!("Failed proposing a new block within timeslot");
                }
            }
        }
    }

    async fn run_proposer(&self) -> Result<(), String> {
        let node = self.clone();
        tokio::spawn(async move {
            node.propose_new_block().await;
        });

        Ok(())
    }
}

#[async_trait]
pub trait Miner {
    async fn run_miner(&self) -> Result<(), String>;
    async fn mine_block(&self);
    async fn mining(&self, block: &mut Block) -> Result<Block, String>;
    async fn send_mined_block(&self, block: Block) -> Result<(), String>;
}

#[async_trait]
impl Miner for Node {
    async fn run_miner(&self) -> Result<(), String> {
        let node = self.clone();
        tokio::spawn(async move {
            node.mine_block().await;
        });

        Ok(())
    }

    async fn mine_block(&self) {
        loop {
            if let Ok(mut block) = self.proposed_block_receiver.recv().await {
                let m_block = self.mining(&mut block).await.unwrap();
                self.send_mined_block(m_block).await.unwrap();
            }
        }
    }

    async fn mining(&self, block: &mut Block) -> Result<Block, String> {
        let block_difficulty = {
            let chain_metadata = ChainMetaData::default();
            chain_metadata.get_block_difficulty().unwrap()
        };

        let mut hash_value = Node::calculate_block_hash(block.clone());
        while !Node::verify_block_hash(hash_value.clone(), block_difficulty) {
            block.inc_nonce();
            hash_value = Node::calculate_block_hash(block.clone());
        }
        block.set_hash(hash_value);
        Ok(block.clone())
    }

    async fn send_mined_block(&self, block: Block) -> Result<(), String> {
        self.mined_block_sender.send(block.clone()).await.unwrap();
        println!("Mined Block: {:?}", block);
        Ok(())
    }
}

#[async_trait]
pub trait Verifier {
    async fn verifier(&self, block: Block) -> bool;
    async fn verify_mined_block(&self);
    async fn run_verifier(&self) -> Result<(), String>;
    async fn add_mind_block_to_chain(&self, block: Block) -> Result<(), String>;
}

#[async_trait]
impl Verifier for Node {
    async fn verifier(&self, block: Block) -> bool {
        let proc_chain = self.chain.write().await;
        if block.prev_hash() != proc_chain.get_leaf().unwrap() {
            return false;
        }

        let hash_value = Node::calculate_block_hash(block.clone());
        if hash_value != block.hash() {
            return false;
        }

        let block_difficulty = {
            let chain_metadata = ChainMetaData::default();
            chain_metadata.get_block_difficulty().unwrap()
        };

        return Node::verify_block_hash(hash_value, block_difficulty)
    }

    async fn verify_mined_block(&self) {
        loop {
            if let Ok(mind_block) = self.mined_block_receiver.recv().await {
                if self.verifier(mind_block.clone()).await {
                    // self.add_mind_block_to_chain(mind_block).await.unwrap();
                    println!("Block Verifying Result: True");
                } else {
                    println!("Block Verifying Result: False");
                }
            }
        }
    }

    async fn run_verifier(&self) -> Result<(), String> {
        let node = self.clone();

        tokio::spawn(async move {
            node.verify_mined_block().await;
        });

        Ok(())
    }

    async fn add_mind_block_to_chain(&self, block: Block) -> Result<(), String> {
        Ok(())
    }
}

#[async_trait]
pub trait NodeController: TxProcesser + Proposer + Miner + Verifier {
    async fn run_node(&self) -> Result<(), String>;
}

#[async_trait]
impl NodeController for Node {
    async fn run_node(&self) -> Result<(), String> {
        let _ = tokio::try_join!(
            async {
                self.run_tx_receiver().await?;
                Ok::<(), String>(())
            },
            async {
                self.run_proposer().await?;
                Ok::<(), String>(())
            },
            async {
                self.run_miner().await?;
                Ok::<(), String>(())
            },
            async {
                self.run_verifier().await?;
                Ok::<(), String>(())
            }
        );

        Ok(())
    }
}
