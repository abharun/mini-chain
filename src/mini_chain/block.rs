use super::transaction::Transaction;
use std::time::{ SystemTime, UNIX_EPOCH };

#[derive(Debug, Clone)]
pub struct Block {
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub prev_hash: String,
    pub hash: String,
}

impl Default for Block {
    fn default() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap().as_secs();
        Self {
            timestamp: timestamp,
            transactions: vec![],
            nonce: 0,
            prev_hash: String::new(),
            hash: String::new(),
        }
    }
}

pub trait BlockOperation {
    fn add_transaction(&self, tx: Transaction) -> bool;
    fn set_prev_hash(&self);
    fn calculate_block_hash(&self);
}

// impl BlockOperation for Block {}
