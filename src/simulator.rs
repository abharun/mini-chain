use crate::client::{Client, TxTriggerController};
use crate::mini_chain::node::NodeController;
use crate::mini_chain::{
    metadata::{ChainMetaData, ChainMetaDataOperation},
    node::Node,
};
use crate::network::{Network, NetworkConfigurer};
use futures::future::try_join_all;

pub async fn chain_simulation() {
    let (node_count, client_count) = {
        let metadata = ChainMetaData::default();
        (
            metadata.get_node_count().unwrap(),
            metadata.get_client_count().unwrap(),
        )
    };

    let mut network = Network::default();

    let mut nodes: Vec<Node> = vec![];
    for _ in 0..node_count {
        let node = Node::new(
            network.get_mined_block_sender(),
            network.get_block_verify_tx_sender(),
        );
        nodes.push(node);
    }

    let mut node_runners = Vec::new();
    for node in &nodes {
        let node = node.clone();
        node_runners.push(async move {
            node.run_node().await?;
            Ok::<(), String>(())
        });
    }

    network.set_pipeline(nodes.clone());

    let mut clients: Vec<Client> = vec![];
    for _ in 0..client_count {
        let client = Client::new(network.get_tx_sender());
        clients.push(client.clone());
    }

    let mut client_runners = Vec::new();
    for client in &clients {
        let client = client.clone();
        client_runners.push(async move {
            client.run_tx_trigger().await;
            Ok::<(), String>(())
        });
    }

    let _ = tokio::try_join!(
        async {
            try_join_all(client_runners).await?;
            Ok::<(), String>(())
        },
        async {
            network.run_network().await?;
            Ok::<(), String>(())
        },
        async {
            try_join_all(node_runners).await?;
            Ok::<(), String>(())
        },
    );
}
