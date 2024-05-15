use super::transaction::Transaction;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Block {
    builder: Option<String>,
    sequence: Option<u64>,
    timestamp: usize,
    tx_count: usize,
    transactions: Vec<Transaction>,
    nonce: usize,
    prev_hash: String,
    hash: String,
}

impl Default for Block {
    fn default() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        Self {
            builder: None,
            sequence: None,
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
    pub fn builder(&self) -> Option<String> { 
        match &self.builder {
            Some(addr) => Some(addr.clone()),
            None => None,
        }
    }
    pub fn sequence(&self) -> Option<u64> { self.sequence }
    pub fn timestamp(&self) -> usize { self.timestamp }
    pub fn tx_count(&self) -> usize { self.tx_count }
    pub fn transactions(&self) -> Vec<Transaction> { self.transactions.clone() }
    pub fn nonce(&self) -> usize { self.nonce }
    pub fn prev_hash(&self) -> String { self.prev_hash.clone() }
    pub fn hash(&self) -> String { self.hash.clone() }
    pub fn inc_nonce(&mut self) { self.nonce += 1; }
    pub fn tx_hashes(&self) -> Vec<String> {
        let hashes: Vec<String> = self.transactions.iter().map(|tx| tx.hash.clone()).collect();
        hashes
    }
}

pub trait BlockConfigurer {
    fn add_transaction(&mut self, tx: Transaction);
    fn set_block_builder(&mut self, addr: String);
    fn set_block_sequence(&mut self, seq: u64);
    fn set_prev_hash(&mut self, prev_hash: String);
    fn set_hash(&mut self, hash: String);
}

impl BlockConfigurer for Block {
    fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.insert(0, tx);
        self.tx_count += 1;
    }

    fn set_block_builder(&mut self, addr: String) {
        self.builder = Some(addr);
    }

    fn set_block_sequence(&mut self, seq: u64) {
        self.sequence = Some(seq);
    }

    fn set_prev_hash(&mut self, prev_hash: String) {
        self.prev_hash = prev_hash;
    }

    fn set_hash(&mut self, hash: String) {
        self.hash = hash;
    }
}
