use super::{chain::Blockchain, mempool::MemPool};

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
    fn build_block(&self) -> Result<(), String>;
}
