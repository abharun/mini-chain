use std::fmt;
use sha3::{Digest, Sha3_256};
use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, PartialEq, Clone)]
pub struct Address(String, String);

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

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

    pub fn get_signature(&self) -> String {
        let mut hasher = Sha3_256::new();

        let data = format!("{}{}", &self.0, &self.1);

        hasher.update(data);

        let hash = format!("{:x}", hasher.finalize());
        hash
    }

    // pub fn get_private_address(&self) -> &str {
    //     &self.1
    // }
}
