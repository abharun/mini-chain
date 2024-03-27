use super::{
    block::{Block, BlockConfigurer},
    chain::{Blockchain, BlockchainOperation},
    mempool::{MemPool, MemPoolOperation},
    metadata::{ChainMetaData, ChainMetaDataOperation},
    transaction::Transaction,
};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use tokio::sync::RwLock;
use std::{sync::Arc, time::{Duration, SystemTime}};

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
    async fn add_tx_to_pool(receiver: Receiver<Transaction>, mempool: Arc<RwLock<MemPool>>);
    async fn run_tx_receiver(&self) -> Result<(), String>;
}

#[async_trait]
impl TxProcesser for Node {
    async fn add_tx_to_pool(receiver: Receiver<Transaction>, mempool: Arc<RwLock<MemPool>>) {
        loop {
            if let Ok(tx) = receiver.recv().await {
                let mut proc_mempool = mempool.write().await;
                let _ = proc_mempool.add_transaction(tx.clone()).await;
                println!("Added tx: {:?}", tx);
            }
        }
    }
    async fn run_tx_receiver(&self) -> Result<(), String> {
        let tx_receiver_thread =
            Self::add_tx_to_pool(self.client_tx_receiver.clone(), self.mempool.clone());

        let _ = tokio::spawn(tx_receiver_thread);

        Ok(())
    }
}

#[async_trait]
pub trait Proposer {
    async fn build_block(&self) -> Result<Block, String> ;
    async fn send_propose_block(&self, block: Block) -> Result<(), String>;
    async fn propose_new_block(&self) -> Result<(), String>;
    async fn run_proposer(&self) -> Result<(), String>;
}

#[async_trait]
impl Proposer for Node {
    async fn build_block(&self) -> Result<Block, String> {
        let (block_propose_time, block_size) = {
            let chain_metadata = ChainMetaData::default();
            (
                chain_metadata.get_block_gen_slot().unwrap(),
                chain_metadata.get_block_size().unwrap(),
            )
        };

        let mut block = Block::default();

        let time_limit = SystemTime::now() + Duration::from_millis(block_propose_time);

        for _ in 0..block_size {
            let mut proc_mempool = self.mempool.write().await;
            let tx = proc_mempool.pickup_transaction().await.unwrap();

            block.add_transaction(tx);

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

    async fn propose_new_block(&self) -> Result<(), String> {
        let block = self.build_block().await?;
        self.send_propose_block(block).await?;
        Ok(())
    }

    async fn run_proposer(&self) -> Result<(), String> {
        loop {
            self.propose_new_block().await?;
            tokio::task::yield_now().await;
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
        loop {
            let _ = tokio::try_join!(self.run_tx_receiver(), self.run_proposer(),)
                .expect_err("Failed to run Node!");
        }
    }
}
