pub struct ChainMetaData {
    node_count: usize,
    client_count: usize,
    tx_gen_slot: usize,
    block_gen_slot: usize,
    block_gen_period: usize,
    block_tx_pickup_period: usize,
    block_size: usize,
    block_difficulty: usize,
}

impl Default for ChainMetaData {
    fn default() -> Self {
        Self {
            node_count: 1,
            client_count: 5,
            tx_gen_slot: 200,
            block_gen_slot: 2000,
            block_gen_period: 500,
            block_tx_pickup_period: 400,
            block_size: 20,
            block_difficulty: 2,
        }
    }
}

pub trait ChainMetaDataOperation {
    fn get_node_count(&self) -> Result<usize, String>;
    fn get_client_count(&self) -> Result<usize, String>;
    fn get_tx_gen_slot(&self) -> Result<usize, String>;
    fn get_block_gen_slot(&self) -> Result<usize, String>;
    fn get_block_gen_period(&self) -> Result<usize, String>;
    fn get_block_tx_pickup_period(&self) -> Result<usize, String>;
    fn get_block_size(&self) -> Result<usize, String>;
    fn get_block_difficulty(&self) -> Result<usize, String>;
}

impl ChainMetaDataOperation for ChainMetaData {
    fn get_block_gen_slot(&self) -> Result<usize, String> {
        Ok(self.block_gen_slot)
    }

    fn get_client_count(&self) -> Result<usize, String> {
        Ok(self.client_count)
    }

    fn get_node_count(&self) -> Result<usize, String> {
        Ok(self.node_count)
    }

    fn get_tx_gen_slot(&self) -> Result<usize, String> {
        Ok(self.tx_gen_slot)
    }

    fn get_block_gen_period(&self) -> Result<usize, String> {
        Ok(self.block_gen_period)
    }

    fn get_block_tx_pickup_period(&self) -> Result<usize, String> {
        Ok(self.block_tx_pickup_period)
    }

    fn get_block_size(&self) -> Result<usize, String> {
        Ok(self.block_size)
    }

    fn get_block_difficulty(&self) -> Result<usize, String> {
        Ok(self.block_difficulty)
    }
}
