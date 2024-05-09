use super::{
    block::{Block, BlockConfigurer},
    chain::{Blockchain, BlockchainOperation},
    mempool::{MemPool, MemPoolOperation},
    metadata::{ChainMetaData, ChainMetaDataOperation},
    transaction::Transaction,
};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
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

    pub net_mined_block_sender: Sender<Block>,

    pub mempool: Arc<RwLock<MemPool>>,
    pub chain: Blockchain,
}

impl Node {
    pub fn new(net_mined_block_sender: Sender<Block>) -> Self {
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

            net_mined_block_sender,

            mempool: Arc::new(RwLock::new(MemPool::default())),
            chain: Blockchain::default(),
        }
    }
}

impl Node {
    fn verify_block_hash(hash: String, difficulty: usize) -> bool {
        let hash_binding = hash.as_str();
        &hash_binding[0..difficulty] == "0".repeat(difficulty)
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

        let prev_hash = self.chain.get_leaf().unwrap();
        block.set_prev_hash(prev_hash);

        block.calculate_block_hash();

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

        while !Node::verify_block_hash(block.hash(), block_difficulty) {
            block.inc_nonce();
            block.calculate_block_hash();
        }
        Ok(block.clone())
    }

    async fn send_mined_block(&self, block: Block) -> Result<(), String> {
        self.net_mined_block_sender.send(block).await.unwrap();
        Ok(())
    }
}

#[async_trait]
pub trait Verifer {
    async fn run_verifier(&self) -> Result<(), String>;
    async fn verify_block(&self);
}

#[async_trait]
impl Verifer for Node {
    async fn run_verifier(&self) -> Result<(), String> {
        let node = self.clone();
        tokio::spawn(async move { 
            node.verify_block().await
        });

        Ok(())
    }

    async fn verify_block(&self) {
        loop {
            if let Ok(block) = self.mined_block_receiver.recv().await {
                println!("Mined Block: {:?}", block);
                // Verify if Block index is over the chain's last index.
                // Verify if TXs are existing in the mempool.
                // Verify if block hash is calculated correctly.
                // Broadcast to other nodes to be appended to the chain.
            }
        }
    }
}

#[async_trait]
pub trait NodeController: TxProcesser + Proposer {
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
