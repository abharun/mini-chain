use async_channel::{Receiver, Sender};

use crate::mini_chain::{
    block::Block,
    node::{BlockVerifyTx, Node},
    transaction::Transaction,
};

#[derive(Debug, Clone)]
pub struct Channels {
    pub tx_sender: Sender<Transaction>,
    pub tx_receiver: Receiver<Transaction>,
    pub node_tx_senders: Vec<Sender<Transaction>>,

    pub mined_block_sender: Sender<Block>,
    pub mined_block_receiver: Receiver<Block>,
    pub node_mined_block_senders: Vec<Sender<Block>>,

    pub block_verify_tx_sender: Sender<BlockVerifyTx>,
    pub block_verify_tx_receiver: Receiver<BlockVerifyTx>,
    pub node_block_verify_tx_senders: Vec<Sender<BlockVerifyTx>>,
}

impl Default for Channels {
    fn default() -> Self {
        let (tx_sender, tx_receiver) = async_channel::unbounded();
        let (mined_block_sender, mined_block_receiver) = async_channel::unbounded();
        let (block_verify_tx_sender, block_verify_tx_receiver) = async_channel::unbounded();
        Self {
            tx_sender,
            tx_receiver,
            node_tx_senders: vec![],

            mined_block_sender,
            mined_block_receiver,
            node_mined_block_senders: vec![],

            block_verify_tx_sender,
            block_verify_tx_receiver,
            node_block_verify_tx_senders: vec![],
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
            self.node_mined_block_senders.push(node.mined_block_sender);
            self.node_block_verify_tx_senders.push(node.block_verify_tx_sender);
        }
    }
}

#[derive(Debug, Clone)]
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
    fn get_mined_block_sender(&self) -> Sender<Block>;
    fn get_block_verify_tx_sender(&self) -> Sender<BlockVerifyTx>;

    fn set_pipeline(&mut self, nodes: Vec<Node>);
}

impl NetworkConfigurer for Network {
    fn get_tx_sender(&self) -> Sender<Transaction> {
        self.channel.tx_sender.clone()
    }

    fn get_mined_block_sender(&self) -> Sender<Block> {
        self.channel.mined_block_sender.clone()
    }

    fn get_block_verify_tx_sender(&self) -> Sender<BlockVerifyTx> {
        self.channel.block_verify_tx_sender.clone()
    }

    fn set_pipeline(&mut self, nodes: Vec<Node>) {
        self.channel.set_pipeline(nodes);
    }
}

impl Network {
    async fn broadcast_message<T: Clone + Send + 'static>(
        receiver: Receiver<T>,
        senders: Vec<Sender<T>>,
    ) {
        loop {
            if let Ok(message) = receiver.recv().await {
                for sender in &senders {
                    let sender = sender.clone();
                    sender.send(message.clone()).await.unwrap();
                }
            }
        }
    }

    async fn run_broadcaster<T: Clone + Send + 'static>(
        &self,
        receiver: Receiver<T>,
        senders: Vec<Sender<T>>,
    ) -> Result<(), String> {
        let broadcast_future = Self::broadcast_message(receiver, senders);
        let _ = tokio::spawn(broadcast_future);

        Ok(())
    }

    pub async fn run_network(&mut self) -> Result<(), String> {
        let _ = tokio::try_join!(
            self.run_broadcaster(
                self.channel.tx_receiver.clone(),
                self.channel.node_tx_senders.clone()
            ),
            self.run_broadcaster(
                self.channel.mined_block_receiver.clone(),
                self.channel.node_mined_block_senders.clone()
            ),
            self.run_broadcaster(
                self.channel.block_verify_tx_receiver.clone(),
                self.channel.node_block_verify_tx_senders.clone()
            )
        );

        Ok(())
    }
}
