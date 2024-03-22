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
            block_gen_slot: 1000,
        }
    }
}

pub trait ChainMetaDataOperation {
    fn get_node_count(&self) -> Result<i32, String>;
    fn get_client_count(&self) -> Result<i32, String>;
    fn get_tx_gen_slot(&self) -> Result<i32, String>;
    fn get_block_gen_slot(&self) -> Result<i32, String>;
}

impl ChainMetaDataOperation for ChainMetaData {
    fn get_block_gen_slot(&self) -> Result<i32, String> {
        Ok(self.block_gen_slot)
    }

    fn get_client_count(&self) -> Result<i32, String> {
        Ok(self.client_count)
    }

    fn get_node_count(&self) -> Result<i32, String> {
        Ok(self.node_count)
    }

    fn get_tx_gen_slot(&self) -> Result<i32, String> {
        Ok(self.tx_gen_slot)
    }
}
