use super::mempool::MemPool;

pub struct Node {
    pub proposed_block_sender: String,
    pub proposed_block_receiver: String,

    pub mined_block_sender: String,
    pub mined_block_receiver: String,

    pub mempool: MemPool
}

impl Node {
    fn new() -> Self {
        Self {
            proposed_block_sender: String::new(),
            proposed_block_receiver: String::new(),

            mined_block_sender: String::new(),
            mined_block_receiver: String::new(),

            mempool: MemPool::default(),
        }
    }
}
