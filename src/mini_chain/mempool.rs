use std::collections::HashMap;

use async_trait::async_trait;

use super::transaction::{Transaction, TxExisting, TxPoolRecord, TxStatus};

#[derive(Debug, Clone)]
pub struct MemPool {
    pub txpool: HashMap<String, TxPoolRecord>,
}

impl Default for MemPool {
    fn default() -> Self {
        Self {
            txpool: HashMap::new(),
        }
    }
}

#[async_trait]
pub trait MemPoolOperation {
    async fn add_transaction(&mut self, tx: Transaction) -> Result<(), String>;
    async fn existing_transaction(&self, tx: Transaction) -> TxExisting;
    async fn pickup_transaction(&mut self, count: usize) -> Result<Vec<Transaction>, String>;
}

#[async_trait]
impl MemPoolOperation for MemPool {
    async fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        self.txpool.insert(
            tx.hash.clone(),
            TxPoolRecord {
                status: TxStatus::RECEIVED,
                transaction: tx.clone(),
            },
        );
        Ok(())
    }

    async fn existing_transaction(&self, tx: Transaction) -> TxExisting {
        match self.txpool.get(&tx.hash) {
            Some(_) => TxExisting::EXISTING,
            None => TxExisting::NONEXISTING,
        }
    }

    async fn pickup_transaction(&mut self, count: usize) -> Result<Vec<Transaction>, String> {
        let pool_received_records: Vec<Transaction> = self
            .txpool
            .iter()
            .filter(|(_key, txrecord)| txrecord.status == TxStatus::RECEIVED)
            .take(count)
            .map(|(_key, record)| record.transaction.clone())
            .collect();

        Ok(pool_received_records)
    }
}
