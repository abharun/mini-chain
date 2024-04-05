use sha3::{Digest, Sha3_256};

use super::transaction::Transaction;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Block {
    timestamp: u64,
    tx_count: u64,
    transactions: Vec<Transaction>,
    nonce: u64,
    prev_hash: String,
    hash: String,
}

impl Default for Block {
    fn default() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            timestamp: timestamp,
            tx_count: 0,
            transactions: vec![],
            nonce: 0,
            prev_hash: String::new(),
            hash: String::new(),
        }
    }
}

impl Block {
    pub fn timestamp(&self) -> u64 { self.timestamp }
    pub fn tx_count(&self) -> u64 { self.tx_count }
    pub fn transactions(&self) -> Vec<Transaction> { self.transactions.clone() }
    pub fn nonce(&self) -> u64 { self.nonce }
    pub fn prev_hash(&self) -> String { self.prev_hash.clone() }
    pub fn hash(&self) -> String { self.hash.clone() }
    pub fn inc_nonce(&mut self) { self.nonce += 1; }
}

pub trait BlockConfigurer {
    fn add_transaction(&mut self, tx: Transaction);
    fn set_prev_hash(&mut self, prev_hash: String);
    fn calculate_block_hash(&mut self);
}

impl BlockConfigurer for Block {
    fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.insert(0, tx);
        self.tx_count += 1;
    }

    fn calculate_block_hash(&mut self) {
        let mut hasher = Sha3_256::new();

        let hash_str = format!("{}{}{}{}", self.timestamp, self.tx_count, self.nonce, self.prev_hash);
        hasher.update(hash_str);

        for tx in &self.transactions {
            let hash_str = format!("{:?}", tx);
            hasher.update(hash_str);
        }

        self.hash = format!("{:x}", hasher.finalize());
    }

    fn set_prev_hash(&mut self, prev_hash: String) {
        self.prev_hash = prev_hash;
    }
}
