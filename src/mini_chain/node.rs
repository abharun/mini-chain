use super::{
    block::{Block, BlockConfigurer},
    chain::{Blockchain, BlockchainOperation},
    mempool::{ MemPool, MemPoolOperation},
    metadata::{ChainMetaData, ChainMetaDataOperation},
    transaction::Transaction,
};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct Node {
    pub client_tx_sender: Sender<Transaction>,
    pub client_tx_receiver: Receiver<Transaction>,

    pub proposed_block_sender: String,
    pub proposed_block_receiver: String,

    pub mined_block_sender: String,
    pub mined_block_receiver: String,

    pub mempool: MemPool,
    pub chain: Blockchain,
}

impl Default for Node {
    fn default() -> Self {
        let (client_tx_sender, client_tx_receiver) = async_channel::unbounded();
        Self {
            client_tx_sender,
            client_tx_receiver,

            proposed_block_sender: String::new(),
            proposed_block_receiver: String::new(),

            mined_block_sender: String::new(),
            mined_block_receiver: String::new(),

            mempool: MemPool::default(),
            chain: Blockchain::default(),
        }
    }
}

#[async_trait]
pub trait TxProcesser {
    async fn add_tx_to_pool(receiver: Receiver<Transaction>, mut mempool: MemPool);
    async fn run_tx_receiver(&self) -> Result<(), String>;
}

#[async_trait]
impl TxProcesser for Node {
    async fn add_tx_to_pool(receiver: Receiver<Transaction>, mut mempool: MemPool) {
        loop {
            if let Ok(tx) = receiver.recv().await {
                mempool.add_transaction(tx.clone()).unwrap();
                println!("Added tx: {:?}", tx);
            }
        }
    }
    async fn run_tx_receiver(&self) -> Result<(), String> {
        let tx_receiver_thread = Self::add_tx_to_pool(self.client_tx_receiver.clone(), self.mempool.clone());

        tokio::spawn(tx_receiver_thread);

        Ok(())
    }
}

pub trait Proposer {
    fn build_block(&mut self) -> Result<Block, String>;
    fn send_propose_block(&mut self) {
        let propose_block = self.build_block().unwrap();
        println!("{:#?}", propose_block);
    }
}

impl Proposer for Node {
    fn build_block(&mut self) -> Result<Block, String> {
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
            let tx = self.mempool.pickup_transaction().unwrap();

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
}
