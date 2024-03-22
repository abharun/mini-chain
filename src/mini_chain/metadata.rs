pub struct ChainMetaData {
    node_count: u64,
    client_count: u64,
    tx_gen_slot: u64,
    block_gen_slot: u64,
    block_size: u64,
}

impl Default for ChainMetaData {
    fn default() -> Self {
        Self {
            node_count: 5,
            client_count: 20,
            tx_gen_slot: 50,
            block_gen_slot: 1000,
            block_size: 20,
        }
    }
}

pub trait ChainMetaDataOperation {
    fn get_node_count(&self) -> Result<u64, String>;
    fn get_client_count(&self) -> Result<u64, String>;
    fn get_tx_gen_slot(&self) -> Result<u64, String>;
    fn get_block_gen_slot(&self) -> Result<u64, String>;
    fn get_block_size(&self) -> Result<u64, String>;
}

impl ChainMetaDataOperation for ChainMetaData {
    fn get_block_gen_slot(&self) -> Result<u64, String> {
        Ok(self.block_gen_slot)
    }

    fn get_client_count(&self) -> Result<u64, String> {
        Ok(self.client_count)
    }

    fn get_node_count(&self) -> Result<u64, String> {
        Ok(self.node_count)
    }

    fn get_tx_gen_slot(&self) -> Result<u64, String> {
        Ok(self.tx_gen_slot)
    }

    fn get_block_size(&self) -> Result<u64, String> {
        Ok(self.block_size)
    }
}
