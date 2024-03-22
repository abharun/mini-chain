use crate::client::Client;
use crate::mini_chain::{
    metadata::{ChainMetaData, ChainMetaDataOperation},
    node::Node,
};
use crate::network::{Network, NetworkConfigurer};

pub fn chain_simulation() {
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

    let network = Network::default();

    let mut clients: Vec<Client> = vec![];
    for _ in 0..client_count {
        let client = Client::new(network.get_tx_sender());
        clients.push(client);
    }

    println!("{:#?}", clients);
}
