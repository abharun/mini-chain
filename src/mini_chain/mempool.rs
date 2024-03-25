use super::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct MemPool {
    pub txqueue: Vec<Transaction>,
}

impl Default for MemPool {
    fn default() -> Self {
        Self { txqueue: vec![] }
    }
}

pub trait MemPoolOperation {
    fn add_transaction(&mut self, tx: Transaction) -> Result<(), String>;
    fn existing_transaction(&self, tx: Transaction) -> Result<bool, String>;
    fn pickup_transaction(&mut self) -> Result<Transaction, String>;
}

impl MemPoolOperation for MemPool {
    fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        self.txqueue.push(tx);
        Ok(())
    }

    fn existing_transaction(&self, tx: Transaction) -> Result<bool, String> {
        Ok(self.txqueue.contains(&tx))
    }

    fn pickup_transaction(&mut self) -> Result<Transaction, String> {
        Ok(self.txqueue.pop().unwrap())
    }
}
