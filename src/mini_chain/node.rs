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

    pub mined_block_sender: String,
    pub mined_block_receiver: String,

    pub mempool: Arc<RwLock<MemPool>>,
    pub chain: Blockchain,
}

impl Default for Node {
    fn default() -> Self {
        let (client_tx_sender, client_tx_receiver) = async_channel::unbounded();
        let (proposed_block_sender, proposed_block_receiver) = async_channel::unbounded();
        Self {
            client_tx_sender,
            client_tx_receiver,

            proposed_block_sender,
            proposed_block_receiver,

            mined_block_sender: String::new(),
            mined_block_receiver: String::new(),

            mempool: Arc::new(RwLock::new(MemPool::default())),
            chain: Blockchain::default(),
        }
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
                println!("Added tx: {:?}", tx);
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

        let time_limit = SystemTime::now() + Duration::from_millis(block_tx_pickup_period);

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
            sleep(Duration::from_millis(block_gen_slot)).await;
            // let block = self.build_block().await;

            let block_builder = self.build_block();

            match tokio::time::timeout(Duration::from_millis(block_gen_period), block_builder).await {
                Ok(Ok(block)) => {
                    println!("Built block for propose: {:?}", block);
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
    async fn receive_proposed_block(&self) -> Result<(), String>;
}

#[async_trait]
impl Miner for Node {
    async fn receive_proposed_block(&self) -> Result<(), String> {
        Ok(())
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
            }
        );

        Ok(())
    }
}
