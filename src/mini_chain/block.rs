use super::transaction::Transaction;

pub struct Block {
    pub timestamp: String,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub prev_hash: String,
    pub hash: String,
}
