use rand::{distributions::Alphanumeric, Rng};

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
