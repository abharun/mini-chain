use async_channel::{Receiver, Sender};

use crate::mini_chain::transaction::Transaction;

pub struct Channels {
    pub client_tx_sender: Sender<Transaction>,
    pub client_tx_receiver: Receiver<Transaction>,
    pub node_tx_senders: Vec<Sender<Transaction>>,
}

impl Default for Channels {
    fn default() -> Self {
        let (client_tx_sender, client_tx_receiver) = async_channel::unbounded();
        Self {
            client_tx_sender,
            client_tx_receiver,
            node_tx_senders: vec![],
        }
    }
}

pub struct Network {
    pub channel: Channels,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            channel: Channels::default(),
        }
    }
}

pub trait NetworkConfigurer {
    fn get_tx_sender(&self) -> Sender<Transaction>;
}

impl NetworkConfigurer for Network {
    fn get_tx_sender(&self) -> Sender<Transaction> {
        self.channel.client_tx_sender.clone()
    }
}
