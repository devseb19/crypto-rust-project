use sled::Db;
use std::str;

pub struct BalanceStore {
    db: Db,
}

impl BalanceStore {
    pub fn new() -> Self {
        let db = sled::open("wallet_db").expect("Failed to open sled DB");
        BalanceStore { db }
    }

    pub fn set_balance(&self, address: &str, amount: u64) -> Result<(), sled::Error> {
        self.db.insert(address, &amount.to_be_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    pub fn get_balance(&self, address: &str) -> Result<u64, sled::Error> {
        if let Some(value) = self.db.get(address)? {
            let bytes: [u8; 8] = value.as_ref().try_into().unwrap_or([0; 8]);
            Ok(u64::from_be_bytes(bytes))
        } else {
            Ok(0)
        }
    }
}
