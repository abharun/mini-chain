#[derive(Debug, PartialEq)]
pub struct Address(String);

#[derive(Debug, PartialEq)]
pub struct Signature(String);

#[derive(Debug, PartialEq)]
pub struct TxPayload {
    pub addr: Address,
    pub amount: u64,
}

#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub timestamp: String,
    pub nonce: u64,
    pub payload: TxPayload,
    pub signer: Address,
    pub signature: Signature,
}
