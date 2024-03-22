use super::transaction::Transaction;

pub struct MemPool {
    pub txqueue: Vec<Transaction>,
}

impl Default for MemPool {
    fn default() -> Self {
        Self {
            txqueue: vec![],
        }
    }
}

trait MemPoolOperation {
    fn add_transaction(&mut self, tx: Transaction) -> Result<(), String>;
    fn existing_transaction(&self, tx: Transaction) -> Result<bool, String>;
}

impl MemPoolOperation for MemPool {
    fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        self.txqueue.push(tx);
        Ok(())
    }

    fn existing_transaction(&self, tx: Transaction) -> Result<bool, String> {
        Ok(self.txqueue.contains(&tx))
    }
}
