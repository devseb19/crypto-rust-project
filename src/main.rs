mod wallet;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};
use std::fmt;

#[derive(Debug, Clone)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f32,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}: {}", self.sender, self.receiver, self.amount)
    }
}

#[derive(Debug)]
struct Block {
    index: u64,
    timestamp: u64,
    previous_hash: String,
    hash: String,
    transactions: Vec<Transaction>,
    nonce: u64,
}

impl Block {
    fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let mut block = Block {
            index,
            timestamp,
            previous_hash,
            hash: String::new(),
            transactions,
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let transactions_data = self.transactions.iter()
            .map(|transaction| transaction.to_string())
            .collect::<Vec<String>>()
            .join("|");
        let contents = format!("{}{}{}{}{}", self.index, self.timestamp, self.previous_hash, transactions_data, self.nonce);
        format!("{:x}", Sha256::digest(contents.as_bytes()))
    }

    fn mine(&mut self, difficulty: usize) {
        while &self.hash[..difficulty] != &"0".repeat(difficulty) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

#[derive(Debug)]
struct Blockchain {
    blocks: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    fn new() -> Self {
        let genesis_block = Block::new(0, "0".to_string(), vec![]);
        Blockchain {
            blocks: vec![genesis_block],
            difficulty: 2, // Just an example difficulty level
        }
    }

    fn add_block(&mut self, transactions: Vec<Transaction>) {
        let previous_block = self.blocks.last().unwrap();
        let mut new_block = Block::new(
            previous_block.index + 1,
            previous_block.hash.clone(),
            transactions,
        );
        new_block.mine(self.difficulty);
        self.blocks.push(new_block);
    }
}
fn main() {
    let keypair = wallet::generate_keypair();
    match keypair {
        Ok((secret_key, public_key)) => {
            let address = wallet::generate_address(&public_key);
            println!("Secret Key: {:?}", secret_key);
            println!("Public Key: {:?}", public_key);
            println!("Address: {}", address);
        },
        Err(e) => println!("Error generating keypair: {:?}", e),
    }

    let mut blockchain = Blockchain::new();

    // Adding a block with a transaction
    blockchain.add_block(vec![
        Transaction {
            sender: "Alice".to_string(),
            receiver: "Bob".to_string(),
            amount: 50.0,
        },
    ]);

    // Adding another block
    blockchain.add_block(vec![
        Transaction {
            sender: "Bob".to_string(),
            receiver: "Charlie".to_string(),
            amount: 25.0,
        },
    ]);

    // Display the blockchain
    println!("Blockchain: {:#?}", blockchain);
}
