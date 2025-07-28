mod wallet;
mod balance;
mod wallet_identity;

use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use balance::BalanceStore;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Mutex;
use wallet_identity::WalletIdentity;

const BLOCKCHAIN_FILE: &str = "./data/blockchain.json";

#[derive(Parser)]
#[command(name = "Crypto Wallet CLI")]
#[command(about = "Blockchain simulation in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    WalletGen {
        #[arg(short, long)]
        name: String,

        #[arg(short, long, default_value_t = 0)]
        fund: u64,
    },
    Balance {
        #[arg(short, long)]
        get: Option<String>,
        #[arg(short, long)]
        set: Option<String>,
        #[arg(short, long)]
        amount: Option<u64>,
    },
    BlockAdd {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: f32,
    },
    BlockchainShow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
        let tx_data = self
            .transactions
            .iter()
            .map(|tx| tx.to_string())
            .collect::<Vec<_>>()
            .join("|");

        let contents = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.previous_hash, tx_data, self.nonce
        );

        format!("{:x}", Sha256::digest(contents.as_bytes()))
    }

    fn mine(&mut self, difficulty: usize) {
        while &self.hash[..difficulty] != &"0".repeat(difficulty) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Blockchain {
    blocks: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    fn new() -> Self {
        let genesis = Block::new(0, "0".to_string(), vec![]);
        Blockchain {
            blocks: vec![genesis],
            difficulty: 2,
        }
    }

    fn add_block(&mut self, transactions: Vec<Transaction>) {
        let prev = self.blocks.last().unwrap();
        let mut new_block = Block::new(prev.index + 1, prev.hash.clone(), transactions);
        new_block.mine(self.difficulty);
        self.blocks.push(new_block);
    }

    fn print_chain(&self) {
        for block in &self.blocks {
            println!("{:#?}", block);
        }
    }

    fn save_to_file(&self, path: &str) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
            {
                let _ = file.write_all(json.as_bytes());
            }
        }
    }

    fn load_from_file(path: &str) -> Self {
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(chain) = serde_json::from_str(&contents) {
                    return chain;
                }
            }
        }
        println!("No existing blockchain found. Creating new chain.");
        Blockchain::new()
    }
}

lazy_static! {
    static ref CHAIN: Mutex<Blockchain> = Mutex::new(Blockchain::load_from_file(BLOCKCHAIN_FILE));
}

fn main() {
    let cli = Cli::parse();
    let store = BalanceStore::new();

    match cli.command {
        Commands::WalletGen { name, fund } => {
            if WalletIdentity::exists(&name) {
                println!("❌ Wallet '{}' already exists.", name);
                return;
            }

            let keypair = wallet::generate_keypair();
            match keypair {
                Ok((secret_key, public_key)) => {
                    let address = wallet::generate_address(&public_key);
                    let identity = WalletIdentity {
                        name: name.clone(),
                        address: address.clone(),
                        public_key: public_key.to_string(),
                        secret_key: secret_key.display_secret().to_string(),
                    };

                    match identity.save_to_file() {
                        Ok(_) => {
                            println!("Wallet '{}' created.", name);
                            println!("Address: {}", address);

                            if fund > 0 {
                                let tx = Transaction {
                                    sender: "Faucet".to_string(),
                                    receiver: name.clone(),
                                    amount: fund as f32,
                                };

                                let mut chain = CHAIN.lock().unwrap();
                                chain.add_block(vec![tx]);
                                chain.save_to_file(BLOCKCHAIN_FILE);

                                let _ = store.set_balance(&address, fund);
                                println!("Funded '{}' with {} coins via block.", name, fund);
                            }
                        }
                        Err(e) => eprintln!("❌ Failed to save wallet: {:?}", e),
                    }
                }
                Err(e) => eprintln!("❌ Keypair generation error: {:?}", e),
            }
        }

        Commands::Balance { get, set, amount } => {
            if let Some(addr) = get {
                let resolved = WalletIdentity::resolve(&addr);
                match store.get_balance(&resolved) {
                    Ok(bal) => println!("Balance for {}: {}", addr, bal),
                    Err(e) => eprintln!("Error: {:?}", e),
                }
            } else if let (Some(addr), Some(amt)) = (set, amount) {
                let resolved = WalletIdentity::resolve(&addr);
                if let Err(e) = store.set_balance(&resolved, amt) {
                    eprintln!("Error setting balance: {:?}", e);
                } else {
                    println!("Set balance for {} to {}", addr, amt);
                }
            } else {
                println!("Usage: balance --get <addr> or --set <addr> --amount <amt>");
            }
        }

        Commands::BlockAdd { from, to, amount } => {
            let amount_u64 = amount.round() as u64;
            let resolved_from = WalletIdentity::resolve(&from);
            let resolved_to = WalletIdentity::resolve(&to);

            let sender_balance = store.get_balance(&resolved_from).unwrap_or(0);
            let receiver_balance = store.get_balance(&resolved_to).unwrap_or(0);

            if sender_balance < amount_u64 {
                println!(
                    "Insufficient funds: {} has {}, needs {}",
                    from, sender_balance, amount_u64
                );
                return;
            }

            let tx = Transaction {
                sender: from.clone(),
                receiver: to.clone(),
                amount,
            };

            let mut chain = CHAIN.lock().unwrap();
            chain.add_block(vec![tx]);
            chain.save_to_file(BLOCKCHAIN_FILE);

            if let Err(e) = store.set_balance(&resolved_from, sender_balance - amount_u64) {
                eprintln!("Error updating sender balance: {:?}", e);
                return;
            }

            if let Err(e) = store.set_balance(&resolved_to, receiver_balance + amount_u64) {
                eprintln!("Error updating receiver balance: {:?}", e);
                return;
            }

            println!("Transaction processed. Block added.");
            println!("{} → {}: {} coins", from, to, amount_u64);
        }

        Commands::BlockchainShow => {
            let chain = CHAIN.lock().unwrap();
            chain.print_chain();
        }
    }
}
