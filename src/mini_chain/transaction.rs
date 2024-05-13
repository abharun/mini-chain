use std::time::{SystemTime, UNIX_EPOCH};

use super::address::Address;

#[derive(Debug, PartialEq, Clone)]
pub struct TxPayload {
    pub addr: Address,
    pub amount: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub timestamp: usize,
    pub nonce: usize,
    pub payload: TxPayload,
    pub signer: Address,
    pub signature: String,
}

impl Transaction {
    pub fn new(addr: Address, amount: usize) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        Self {
            timestamp: timestamp,
            nonce: 0,
            payload: TxPayload {
                addr: addr,
                amount: amount,
            },
            signer: Address(String::new(), String::new()),
            signature: String::new(),
        }
    }
}
