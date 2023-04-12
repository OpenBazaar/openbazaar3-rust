use bdk::keys::bip39::{Language, Mnemonic};

use libp2p::core::identity::ed25519;
use std::str::FromStr;

pub fn generate_mnemonic() -> String {
    let mnemonic: Mnemonic = Mnemonic::generate_in(Language::English, 12).unwrap();
    mnemonic.to_string()
}

pub fn generate_keypair_from_mnemonic(
    mnemonic_str: &String,
) -> Result<libp2p::identity::Keypair, Box<dyn std::error::Error>> {
    let mnemonic = Mnemonic::from_str(&mnemonic_str)?;

    let seed_64_bytes = mnemonic.to_seed("");

    // Truncate the 64-bytes to 32-bytes
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&seed_64_bytes[0..32]);

    let sk = ed25519::SecretKey::from_bytes(seed).expect("not the right amount of bytes");
    let keypair: libp2p::identity::Keypair =
        libp2p::identity::Keypair::Ed25519(ed25519::Keypair::from(sk));

    Ok(keypair)
}
