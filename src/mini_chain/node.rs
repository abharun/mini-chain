use super::{
    address::Address,
    block::{Block, BlockConfigurer},
    chain::{Blockchain, BlockchainOperation},
    mempool::{MemPool, MemPoolOperation},
    metadata::{ChainMetaData, ChainMetaDataOperation},
    transaction::{Transaction, TxExisting},
};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use sha3::{Digest, Sha3_256};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::RwLock,
    time::{sleep, timeout},
};

#[derive(Debug, Clone)]
pub struct BlockVerifyTx {
    pub block_hash: String,
    pub verified: bool,
}

#[derive(Debug, Clone)]
struct StagedBlockStatus {
    block: Block,
    handsup: u64,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub address: Address,

    pub client_tx_sender: Sender<Transaction>,
    pub client_tx_receiver: Receiver<Transaction>,

    pub proposed_block_sender: Sender<Block>,
    pub proposed_block_receiver: Receiver<Block>,

    pub mined_block_sender: Sender<Block>,
    pub mined_block_receiver: Receiver<Block>,

    pub block_verify_tx_sender: Sender<BlockVerifyTx>,
    pub block_verify_tx_receiver: Receiver<BlockVerifyTx>,

    pub net_mined_block_sender: Sender<Block>,
    pub net_block_verify_tx_sender: Sender<BlockVerifyTx>,

    stagepool: Arc<RwLock<HashMap<String, StagedBlockStatus>>>,
    mempool: Arc<RwLock<MemPool>>,
    chain: Arc<RwLock<Blockchain>>,
}

impl Node {
    pub fn new(
        net_mined_block_sender: Sender<Block>,
        net_block_verify_tx_sender: Sender<BlockVerifyTx>,
    ) -> Self {
        let address = Address::new();
        let (client_tx_sender, client_tx_receiver) = async_channel::unbounded();
        let (proposed_block_sender, proposed_block_receiver) = async_channel::unbounded();
        let (mined_block_sender, mined_block_receiver) = async_channel::unbounded();
        let (block_verify_tx_sender, block_verify_tx_receiver) = async_channel::unbounded();
        Self {
            address,

            client_tx_sender,
            client_tx_receiver,

            proposed_block_sender,
            proposed_block_receiver,

            mined_block_sender,
            mined_block_receiver,

            block_verify_tx_sender,
            block_verify_tx_receiver,

            net_mined_block_sender,
            net_block_verify_tx_sender,

            mempool: Arc::new(RwLock::new(MemPool::default())),
            chain: Arc::new(RwLock::new(Blockchain::default())),
            stagepool: Arc::new(RwLock::new(HashMap::new())),
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

        let hash_str = format!(
            "{}{}{}{}{}{}",
            block.builder().unwrap(),
            block.sequence().unwrap(),
            block.timestamp(),
            block.tx_count(),
            block.nonce(),
            block.prev_hash()
        );
        hasher.update(hash_str);

        for tx in block.transactions() {
            let hash_str = format!("{:?}", tx);
            hasher.update(hash_str);
        }

        let hash = format!("{:x}", hasher.finalize());
        hash
    }
}

// Receive TXs from Clients and store it into Mempool.
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

// Choose TXs from Mempool and build a new block.
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

        block.set_block_builder(self.address.get_public_address().to_string());

        let proc_chain = self.chain.write().await;
        block.set_block_sequence(proc_chain.get_sequence().unwrap());

        let mut proc_mempool = self.mempool.write().await;

        match timeout(
            Duration::from_millis(block_tx_pickup_period as u64),
            async { proc_mempool.pickup_transaction(block_size).await },
        )
        .await
        {
            Ok(Ok(transactions)) => {
                for tx in transactions.iter() {
                    block.add_transaction(tx.clone());
                }
            }
            Ok(Err(_)) => {}
            Err(_) => {}
        };

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

// Receives a proposed block and mine it by calculating block hash.
#[async_trait]
pub trait Miner {
    async fn run_miner(&mut self) -> Result<(), String>;
    async fn mine_block(&mut self);
    async fn mining(&self, block: &mut Block) -> Result<Block, String>;
    async fn send_mined_block(&mut self, block: Block) -> Result<(), String>;
}

#[async_trait]
impl Miner for Node {
    async fn run_miner(&mut self) -> Result<(), String> {
        let mut node = self.clone();
        tokio::spawn(async move {
            node.mine_block().await;
        });

        Ok(())
    }

    async fn mine_block(&mut self) {
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

    async fn send_mined_block(&mut self, block: Block) -> Result<(), String> {
        let mut proc_stagepool = self.stagepool.write().await;
        proc_stagepool.insert(
            block.hash().clone(),
            StagedBlockStatus {
                block: block.clone(),
                handsup: 1,
            },
        );
        self.net_mined_block_sender
            .send(block.clone())
            .await
            .unwrap();
        Ok(())
    }
}

// Receives a mined block and verify it if it's valid block. If it's verified, add it to the chain.
#[async_trait]
pub trait Verifier {
    async fn verifier(&self, block: Block) -> bool;
    async fn verify_mined_block(&mut self);
    async fn run_verifier(&self) -> Result<(), String>;
}

#[async_trait]
impl Verifier for Node {
    async fn verifier(&self, block: Block) -> bool {
        let proc_chain = self.chain.write().await;
        if block.prev_hash() != proc_chain.get_leaf().unwrap() {
            return false;
        }

        if block.sequence().unwrap() != proc_chain.get_sequence().unwrap() {
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

        let proc_pool = self.mempool.write().await;
        for tx in block.transactions() {
            if proc_pool.existing_transaction(tx.clone()).await == TxExisting::NONEXISTING {
                return false;
            }
        }

        return Node::verify_block_hash(hash_value, block_difficulty);
    }

    async fn verify_mined_block(&mut self) {
        loop {
            if let Ok(mined_block) = self.mined_block_receiver.recv().await {
                if mined_block.builder() == Some(self.address.get_public_address().to_string()) {
                    continue;
                }

                let mut proc_stagepool = self.stagepool.write().await;
                proc_stagepool.insert(
                    mined_block.hash().clone(),
                    StagedBlockStatus {
                        block: mined_block.clone(),
                        handsup: 1,
                    },
                );

                if self.verifier(mined_block.clone()).await {
                    let _ = self
                        .net_block_verify_tx_sender
                        .send(BlockVerifyTx {
                            block_hash: mined_block.hash().clone(),
                            verified: true,
                        })
                        .await;
                } else {
                    let _ = self
                        .net_block_verify_tx_sender
                        .send(BlockVerifyTx {
                            block_hash: mined_block.hash().clone(),
                            verified: false,
                        })
                        .await;
                }
            }
        }
    }

    async fn run_verifier(&self) -> Result<(), String> {
        let mut node = self.clone();

        tokio::spawn(async move {
            node.verify_mined_block().await;
        });

        Ok(())
    }
}

#[async_trait]
pub trait ChainManager {
    async fn run_chain_manager(&self) -> Result<(), String>;
    async fn chain_manager(&mut self);
    async fn add_block_to_chain(&self) -> Result<(), String>;
}

#[async_trait]
impl ChainManager for Node {
    async fn add_block_to_chain(&self) -> Result<(), String> {
        Ok(())
    }

    async fn chain_manager(&mut self) {
        loop {
            if let Ok(block_verify_tx) = self.block_verify_tx_receiver.recv().await {
                let mut proc_stagepool = self.stagepool.write().await;
                if let Some(prev_block_status) = proc_stagepool.get_mut(&block_verify_tx.block_hash)
                {
                    prev_block_status.handsup += 1;

                    let mut proc_chain = self.chain.write().await;
                    if prev_block_status.handsup > (proc_chain.get_sequence().unwrap() * 2 / 3) {
                        let prev_status =
                            proc_stagepool.remove(&block_verify_tx.block_hash).unwrap();
                        let _ = proc_chain.add_block(prev_status.block.clone());

                        let mut proc_mempool = self.mempool.write().await;
                        let _ = proc_mempool.remove_transactions(prev_status.block.tx_hashes().clone()).await;
                    }
                } else {
                    // If staged block is not exisiting on StagePool
                    // Request to get the block to other nodes.
                }
            }
        }
    }

    async fn run_chain_manager(&self) -> Result<(), String> {
        let mut node = self.clone();

        tokio::spawn(async move {
            node.chain_manager().await;
        });

        Ok(())
    }
}

// Whole Node Controller
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
                let mut node = self.clone();
                node.run_miner().await?;
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
