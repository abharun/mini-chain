use super::transaction::Transaction;

pub struct MemPool {
    pub txqueue: Vec<Transaction>,
}
