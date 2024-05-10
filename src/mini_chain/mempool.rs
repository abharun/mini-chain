use async_trait::async_trait;

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

#[async_trait]
pub trait MemPoolOperation {
    async fn add_transaction(&mut self, tx: Transaction) -> Result<(), String>;
    async fn existing_transaction(&self, tx: Transaction) -> Result<bool, String>;
    async fn pickup_transaction(&mut self) -> Result<Transaction, String>;
}

#[async_trait]
impl MemPoolOperation for MemPool {
    async fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        self.txqueue.push(tx);
        Ok(())
    }

    async fn existing_transaction(&self, tx: Transaction) -> Result<bool, String> {
        Ok(self.txqueue.contains(&tx))
    }

    async fn pickup_transaction(&mut self) -> Result<Transaction, String> {
        if self.txqueue.len() == 0 {
            return Err(String::from("Failed to pickup transaction from mempool."));
        }
        let tx = self.txqueue.remove(0);
        Ok(tx)
    }
}
