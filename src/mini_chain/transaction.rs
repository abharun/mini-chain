use std::fmt;
use sha3::{Digest, Sha3_256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq, Clone)]
pub struct TxPayload {
    pub addr: String,
    pub amount: usize,
}

impl fmt::Display for TxPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.addr, self.amount)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub timestamp: usize,
    pub nonce: usize,
    pub payload: TxPayload,
    pub signer: String,
    pub signature: String,
    pub hash: String,
}

impl Transaction {
    pub fn new(to_addr: String, amount: usize) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        Self {
            timestamp: timestamp,
            nonce: 0,
            payload: TxPayload {
                addr: to_addr.clone(),
                amount: amount,
            },
            signer: String::new(),
            signature: String::new(),
            hash: String::new(),
        }
    }

    pub fn sign_transaction(&self, addr: String, signature: String) {
        self.signer = addr.clone();
        self.signature = signature.clone();
    }

    pub fn calculate_hash(tx: Transaction) -> String {
        let mut hasher = Sha3_256::new();

        let data = format!(
            "{}{}{}{}{}",
            tx.timestamp, tx.nonce, tx.payload, tx.signer, tx.signature
        );

        hasher.update(data);

        let hash = format!("{:x}", hasher.finalize());
        hash
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TxPoolRecord {
    pub status: String,
    pub transaction: Transaction,
}
