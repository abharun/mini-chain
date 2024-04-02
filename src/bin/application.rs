use mini_blockchain::simulator::chain_simulation;

#[tokio::main]
async fn main() {
    chain_simulation().await;
}
