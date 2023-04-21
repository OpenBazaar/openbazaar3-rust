use crate::crypto::{self, generate_mnemonic};
use crate::profile::Profile;
use async_trait::async_trait;
use sled;

#[async_trait]
pub trait DB {
    async fn new(db_file: String) -> anyhow::Result<Self>
    where
        Self: Sized;
    async fn get_identity(&self) -> anyhow::Result<libp2p::identity::Keypair>;
    async fn get_mnemonic(&self) -> anyhow::Result<String>;
    async fn save_message(&self, address: &[u8], content: &[u8])
        -> anyhow::Result<Option<Vec<u8>>>;
    async fn get_message(&self, address: &[u8]) -> anyhow::Result<Option<Vec<u8>>>;
    async fn remove_message(&self, address: &[u8]) -> anyhow::Result<Option<Vec<u8>>>;
    async fn get_profile(&self) -> anyhow::Result<Option<crate::profile::Profile>>;
    async fn set_profile(&self, profile: &Profile) -> anyhow::Result<()>;
}
#[derive(Clone, Debug)]
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
        let mnemonic = self.get_mnemonic().await.unwrap();
        let kp = crypto::generate_keypair_from_mnemonic(&mnemonic).unwrap();

        Ok(kp)
    }

    async fn get_mnemonic(&self) -> anyhow::Result<String> {
        let identity = self.db.get(b"identity").expect("Failed to fetch value");
        let mnemonic;

        if let None = identity {
            println!("No identity found in db");
            mnemonic = generate_mnemonic();
            self.db.insert(b"identity", mnemonic.as_bytes()).unwrap();
        } else {
            println!("Identity found in db");
            mnemonic = String::from_utf8(identity.unwrap().to_vec()).unwrap();
        }

        Ok(mnemonic)
    }

    async fn save_message(
        &self,
        address: &[u8],
        content: &[u8],
    ) -> anyhow::Result<Option<Vec<u8>>> {
        match self.db.insert(address, content) {
            Ok(d) => Ok(d.map(|d| d.to_vec())),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    async fn get_message(&self, address: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        let message = self.db.get(address)?;
        Ok(message.map(|e| e.to_vec()))
    }

    async fn remove_message(&self, address: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
        let prev_val = self.db.remove(address)?;
        Ok(prev_val.map(|e| e.to_vec()))
    }

    async fn get_profile(&self) -> anyhow::Result<Option<Profile>> {
        let profile = self.db.get(b"profile")?;
        Ok(profile.map(|e| bincode::deserialize(&e.to_vec()).unwrap()))
    }

    async fn set_profile(&self, profile: &Profile) -> anyhow::Result<()> {
        self.db
            .insert(b"profile", bincode::serialize(profile).unwrap())?;
        Ok(())
    }
}
