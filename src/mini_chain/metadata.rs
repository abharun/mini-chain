pub struct ChainMetaData {
    node_count: i32,
    client_count: i32,
    tx_gen_slot: i32,
    block_gen_slot: i32,
}

impl Default for ChainMetaData {
    fn default() -> Self {
        Self {
            node_count: 5,
            client_count: 20,
            tx_gen_slot: 100,
            block_gen_slot: 1000
        }
    }
}
