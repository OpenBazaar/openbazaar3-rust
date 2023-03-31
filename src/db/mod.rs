use crate::crypto::{self, generate_mnemonic};
use async_trait::async_trait;
use sled;

#[async_trait]
pub trait DB {
    async fn new(db_file: String) -> anyhow::Result<Self>
    where
        Self: Sized;
    async fn get_identity(&self) -> anyhow::Result<libp2p::identity::Keypair>;
}

pub struct OpenBazaarDb {
    pub db: sled::Db,
}

#[async_trait]
impl DB for OpenBazaarDb {
    async fn new(db_file: String) -> anyhow::Result<Self> {
        let db: sled::Db = sled::open(db_file).unwrap();
        Ok(OpenBazaarDb { db })
    }

    async fn get_identity(&self) -> anyhow::Result<libp2p::identity::Keypair> {
        let identity = self.db.get(b"identity").expect("Failed to fetch value");
        let mut mnemonic = String::new();

        if let None = identity {
            println!("No identity found in db");
            mnemonic = generate_mnemonic();
            self.db.insert(b"identity", mnemonic.as_bytes()).unwrap();
        } else {
            println!("Identity found in db");
            mnemonic = String::from_utf8(identity.unwrap().to_vec()).unwrap();
        }

        let kp = crypto::generate_keypair_from_mnemonic(&mnemonic).unwrap();

        Ok(kp)
    }
}
