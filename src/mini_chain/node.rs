use super::{
    block::{Block, BlockConfigurer},
    chain::{Blockchain, BlockchainOperation},
    mempool::{MemPool, MemPoolOperation},
    metadata::{ChainMetaData, ChainMetaDataOperation},
};
use std::time::{Duration, SystemTime};

pub struct Node {
    pub proposed_block_sender: String,
    pub proposed_block_receiver: String,

    pub mined_block_sender: String,
    pub mined_block_receiver: String,

    pub mempool: MemPool,
    pub chain: Blockchain,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            proposed_block_sender: String::new(),
            proposed_block_receiver: String::new(),

            mined_block_sender: String::new(),
            mined_block_receiver: String::new(),

            mempool: MemPool::default(),
            chain: Blockchain::default(),
        }
    }
}

pub trait Proposer {
    fn build_block(&mut self) -> Result<Block, String>;
    fn send_propose_block(&mut self) {
        let _propose_block = self.build_block().unwrap();
        // Send proposed block via sender
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
