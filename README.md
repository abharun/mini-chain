# mini-POW blockchain simulation

## Overview

Proof of work (PoW) is a decentralized consensus mechanism that requires network members to expend effort in solving an encryption puzzle.roof of work is also called mining, in reference to receiving a reward for work done.

To mine a new block, each node should calculate a hash value that satisfy the condition under fixed range of amount.

This application simulate general functionalities for components - clients, nodes and the network - in POW network.

## How to run

Clone the code base.

```bash
git clone https://github.com/abharun/mini-chain.git
```

Then you can run the application with `cargo run` simply.

By changing the chain metadata values in `src/mini_chain/metadata.rs`, you can modify the network performance and check how it works.

```rust
impl Default for ChainMetaData {
    fn default() -> Self {
        Self {
            node_count: 1,                  // number of nodes
            client_count: 5,                // number of clients that trigger transactions
            tx_gen_slot: 200,               // time period each client trigger a new transaction
            block_gen_slot: 2000,           // time till next block is generated (not mined yet)
            block_gen_period: 500,          // time limit for building a block
            block_tx_pickup_period: 400,    // time limit for collecting transactions from mempool
            block_size: 20,                 // maximun number of transactions in one block
            block_difficulty: 2,            // current block generation difficulty
        }
    }
}
```

## Structures

### Chain Metadata

```rust
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
```

### Components

```rust
// Client
pub struct Client {
    addr: Address,
    net_tx_sender: Sender<Transaction>,
}
```

```rust
// MemPool
pub struct MemPool {
    pub txpool: HashMap<String, TxPoolRecord>,
}
```

```rust
// Node
pub struct Node {
    pub address: Address,

    pub client_tx_sender: Sender<Transaction>,
    pub client_tx_receiver: Receiver<Transaction>,

    pub proposed_block_sender: Sender<Block>,
    pub proposed_block_receiver: Receiver<Block>,

    pub mined_block_sender: Sender<Block>,
    pub mined_block_receiver: Receiver<Block>,

    pub block_verify_tx_sender: Sender<BlockVerifyTx>,
    pub block_verify_tx_receiver: Receiver<BlockVerifyTx>,

    non_existing_block_sender: Sender<StagedBlockStatus>,
    non_existing_block_receiver: Receiver<StagedBlockStatus>,

    pub non_existing_block_request_sender: Sender<GetNonExistingBlockTx>,
    pub non_existing_block_request_receiver: Receiver<GetNonExistingBlockTx>,

    pub net_mined_block_sender: Sender<Block>,
    pub net_block_verify_tx_sender: Sender<BlockVerifyTx>,
    pub net_non_existing_block_request_sender: Sender<GetNonExistingBlockTx>,

    stagepool: Arc<RwLock<HashMap<String, StagedBlockStatus>>>,
    mempool: Arc<RwLock<MemPool>>,
    chain: Arc<RwLock<Blockchain>>,
}
```

```rust
// Chain
pub struct Blockchain {
    blocks: HashMap<String, Block>,
    leaf: String,
    sequence: u64
}
```

### Data formats

```rust
// Address
pub struct Address(String, String);     // (public address, private address)
```

```rust
// Transaction
pub struct TxPayload {
    pub addr: String,
    pub amount: usize,
}

pub struct Transaction {
    pub timestamp: usize,
    pub nonce: usize,
    pub payload: TxPayload,
    pub signer: String,
    pub signature: String,
    pub hash: String,
}
```

```rust
// Block
pub struct Block {
    builder: Option<String>,
    sequence: Option<u64>,
    timestamp: usize,
    tx_count: usize,
    transactions: Vec<Transaction>,
    nonce: usize,
    prev_hash: String,
    hash: String,
}
```

### Network

```rust
// Channels in network
pub struct Channels {
    pub tx_sender: Sender<Transaction>,
    pub tx_receiver: Receiver<Transaction>,
    pub node_tx_senders: Vec<Sender<Transaction>>,

    pub mined_block_sender: Sender<Block>,
    pub mined_block_receiver: Receiver<Block>,
    pub node_mined_block_senders: Vec<Sender<Block>>,

    pub block_verify_tx_sender: Sender<BlockVerifyTx>,
    pub block_verify_tx_receiver: Receiver<BlockVerifyTx>,
    pub node_block_verify_tx_senders: Vec<Sender<BlockVerifyTx>>,

    pub non_existing_block_request_sender: Sender<GetNonExistingBlockTx>,
    pub non_existing_block_request_receiver: Receiver<GetNonExistingBlockTx>,
    pub node_non_existing_block_request_senders: Vec<Sender<GetNonExistingBlockTx>>,
}

pub struct Network {
    pub channel: Channels,
}
```

## How does it work

On the ground, the `Network` app that involves all the nodes, clients and entrypoints for transfor data is running.

All the clients and nodes are connected to the Network, and all the transactions and blocks come and go via the network.

### Transaction flow

Trigger a transaction -> Send TX to Network -> Broadcast to Nodes -> Stored in Mempools within Nodes.

### Mining a block

Pick up TXs from Mempool -> Build Block include TXs -> Mine Block with difficulty -> Send mined Block to Network -> Broadcast to Nodes

### Verify mined block

Receive mined block -> Validate block hash -> Send Block verification TX to Network -> Broadcast verification TX to Nodes -> Check BFT -> Add block to chain
