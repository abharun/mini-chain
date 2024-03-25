use async_channel::{Receiver, Sender};

use crate::mini_chain::{node::Node, transaction::Transaction};

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

pub trait ChannelConfigurer {
    fn set_pipeline(&mut self, nodes: Vec<Node>);
}

impl ChannelConfigurer for Channels {
    fn set_pipeline(&mut self, nodes: Vec<Node>) {
        for node in nodes {
            self.node_tx_senders.push(node.client_tx_sender);
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

    fn set_pipeline(&mut self, nodes: Vec<Node>);
}

impl NetworkConfigurer for Network {
    fn get_tx_sender(&self) -> Sender<Transaction> {
        self.channel.client_tx_sender.clone()
    }

    fn set_pipeline(&mut self, nodes: Vec<Node>) {
        self.channel.set_pipeline(nodes);
    }
}

impl Network {
    pub async fn broadcast_message<T: Clone + Send + 'static>(
        receiver: Receiver<T>,
        senders: Vec<Sender<T>>,
    ) {
        let message = receiver.recv().await.unwrap();

        for sender in senders {
            let sender = sender.clone();
            sender.send(message.clone()).await.unwrap();
        }
    }

    pub async fn display_received_tx(receiver: Receiver<Transaction>) {
        let tx = receiver.recv().await.unwrap();
        println!("{:#?}", tx);
    }

    pub async fn run_network(&mut self) -> Result<(), String> {
        let broadcast_future = Self::broadcast_message(
            self.channel.client_tx_receiver.clone(),
            self.channel.node_tx_senders.clone(),
        );

        let _ = tokio::spawn(broadcast_future);

        Ok(())
    }

    pub async fn tx_receiver(&mut self) -> Result<(), String> {
        let display_tx_future = Self::display_received_tx(self.channel.client_tx_receiver.clone());

        let _ = tokio::spawn(display_tx_future);

        Ok(())
    }
}
