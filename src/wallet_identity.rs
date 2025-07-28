use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletIdentity {
    pub name: String,
    pub address: String,
    pub public_key: String,
    pub secret_key: String,
}

impl WalletIdentity {
    pub fn resolve(input: &str) -> String {
        let path = format!("./wallets/{}.json", input);
        if Path::new(&path).exists() {
            if let Ok(mut file) = File::open(&path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(identity) = serde_json::from_str::<WalletIdentity>(&contents) {
                        return identity.address;
                    }
                }
            }
        }
        input.to_string()
    }

    pub fn exists(name: &str) -> bool {
        Path::new("wallets").join(format!("{}.json", name)).exists()
    }

    pub fn save_to_file(&self) -> std::io::Result<()> {
        fs::create_dir_all("wallets")?;
        let path = format!("./wallets/{}.json", self.name);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}
