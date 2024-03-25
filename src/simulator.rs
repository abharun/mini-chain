use crate::client::{Client, TxTriggerController};
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

    let mut nodes: Vec<Node> = vec![];
    for _ in 0..node_count {
        let node = Node::default();
        nodes.push(node);
    }

    let mut network = Network::default();

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

    // let _ = try_join_all(client_runners).await;
    let _ = tokio::try_join!(
        async {
            try_join_all(client_runners).await?;
            Ok::<(), String>(())
        },
        async {
            network.tx_receiver().await?;
            Ok::<(), String>(())
        }
    );
}
