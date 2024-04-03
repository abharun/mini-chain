use std::time::Duration;

use crate::mini_chain::{
    metadata::{ChainMetaData, ChainMetaDataOperation},
    transaction::{Address, Transaction},
};
use async_channel::Sender;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Client {
    addr: Address,
    net_tx_sender: Sender<Transaction>,
}

impl Client {
    pub fn new(tx_sender: Sender<Transaction>) -> Self {
        let new_addr = Address::new();

        Self {
            addr: new_addr,
            net_tx_sender: tx_sender,
        }
    }
}

#[async_trait]
pub trait TxTrigger {
    async fn rand_tx_trigger(&self) -> Result<(), String>;
}

#[async_trait]
impl TxTrigger for Client {
    async fn rand_tx_trigger(&self) -> Result<(), String> {
        // let mut rnd = rand::thread_rng();
        // let amount = rnd.gen_range(0..100);
        let amount = 20;

        let new_tx = Transaction::new(self.addr.clone(), amount);

        let _ = self.net_tx_sender.send(new_tx).await.map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[async_trait]
pub trait TxTriggerController: TxTrigger {
    async fn run_tx_trigger(&self) {
        let metadata = ChainMetaData::default();
        let tx_trigger_slot = metadata.get_tx_gen_slot().unwrap();
        loop {
            let _ = self.rand_tx_trigger().await;
            tokio::time::sleep(Duration::from_millis(tx_trigger_slot)).await;
        }
    }
}

impl TxTriggerController for Client {}
