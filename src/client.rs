use crate::mini_chain::transaction::{Address, Transaction};
use async_channel::Sender;

#[derive(Debug, Clone)]
pub struct Client {
    addr: Address,
    amount: u64,
    net_tx_sender: Sender<Transaction>,
}

impl Client {
    pub fn new(tx_sender: Sender<Transaction>) -> Self {
        let new_addr = Address::new();

        Self {
            addr: new_addr,
            amount: 0,
            net_tx_sender: tx_sender,
        }
    }
}
