use super::transaction::Transaction;

pub struct Block {
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub prev_hash: String,
    pub hash: String,
}
