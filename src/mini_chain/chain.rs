use super::block::Block;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Blockchain {
    blocks: HashMap<String, Block>,
    leaf: String,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self {
            blocks: HashMap::new(),
            leaf: String::new(),
        }
    }
}

pub trait BlockchainOperation {
    fn add_block(&mut self, block: Block) -> Result<(), String>;
    fn get_leaf(&self) -> Result<String, String>;
}

impl BlockchainOperation for Blockchain {
    fn add_block(&mut self, block: Block) -> Result<(), String> {
        if self.blocks.contains_key(&block.prev_hash) || self.blocks.is_empty() {
            self.blocks.insert(block.hash.clone(), block.clone());
            self.leaf = block.hash.clone();
        }

        Ok(())
    }

    fn get_leaf(&self) -> Result<String, String> {
        Ok(self.leaf.clone())
    }
}
