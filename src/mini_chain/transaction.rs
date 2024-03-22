use rand::{distributions::Alphanumeric, Rng};
use sha3::{Digest, Sha3_256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq, Clone)]
pub struct Address(String, String);

impl Address {
    pub fn new() -> Self {
        let pub_addr: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect();

        let pri_addr: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        Address(pub_addr, pri_addr)
    }

    pub fn get_public_address(&self) -> &str {
        &self.0
    }

    pub fn get_private_address(&self) -> &str {
        &self.1
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Signature(String);

impl Signature {
    pub fn generate_signature(addr: Address) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(addr.get_private_address());
        let sign = format!("{:x}", hasher.finalize());
        Signature(sign)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TxPayload {
    pub addr: Address,
    pub amount: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub timestamp: u64,
    pub nonce: u64,
    pub payload: TxPayload,
    pub signer: Address,
    pub signature: Signature,
}

impl Transaction {
    pub fn new(addr: Address, amount: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            timestamp: timestamp,
            nonce: 0,
            payload: TxPayload {
                addr: addr,
                amount: amount,
            },
            signer: Address(String::new(), String::new()),
            signature: Signature(String::new()),
        }
    }

    pub fn sign_transaction(&mut self, addr: Address) -> Result<(), String> {
        self.signer = addr.clone();
        self.signature = Signature::generate_signature(addr.clone());

        Ok(())
    }
}
