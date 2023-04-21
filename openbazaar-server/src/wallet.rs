use bdk::bitcoin::Network;
use bdk::keys::bip39::Mnemonic;
use bdk::keys::{DerivableKey, ExtendedKey};
use bdk::template::Bip84;
use bdk::wallet::AddressIndex;
use bdk::Wallet;
// use bdk_esplora::{esplora_client, EsploraExt};
use bdk_file_store::KeychainStore;
// use std::collections::BTreeMap;
// use std::io::Write;

// const SEND_AMOUNT: u64 = 5000;
// const STOP_GAP: usize = 50;
// const PARALLEL_REQUESTS: usize = 5;

pub fn fire_up_wallet(mnemonic_words: String, data_dir: String) {
    let network = Network::Testnet;

    let mnemonic = Mnemonic::parse(&mnemonic_words).unwrap();

    let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
    let xpriv = xkey.into_xprv(network).unwrap();

    // Create the data folder if doesn't exist
    std::fs::create_dir_all(&data_dir).expect("Failed to create data folder");

    let db = KeychainStore::new_from_path(format!("{}/wallet.db", &data_dir))
        .expect("Failed to create keychain store");

    let mut wallet = Wallet::new(
        Bip84(xpriv.clone(), bdk::KeychainKind::External),
        Some(Bip84(xpriv, bdk::KeychainKind::Internal)),
        db,
        network,
    )
    .unwrap();

    // println!(
    //     "mnemonic: {}\n\nrecv desc (pub key): {:#?}\n\nchng desc (pub key): {:#?}",
    //     mnemonic_words,
    //     wallet.get_descriptor_for_keychain(bdk::KeychainKind::External),
    //     wallet.get_descriptor_for_keychain(bdk::KeychainKind::Internal),
    // );

    println!(
        "Revealed address: {}",
        wallet.get_address(AddressIndex::New)
    );
    wallet
        .commit()
        .expect("couldn't save new address to the wallet");
    let balance = wallet.get_balance();
    println!("Balance: {} sats", balance);

    // print!("Syncing...");
    // // Scanning the chain...
    // let esplora_url = "https://mempool.space/testnet/api";
    // let client = esplora_client::Builder::new(esplora_url)
    //     .build_blocking()
    //     .expect("something is screwed with esplora");
    // let checkpoints = wallet.checkpoints();
    // let spks: BTreeMap<_, _> = wallet
    //     .spks_of_all_keychains()
    //     .into_iter()
    //     .map(|(k, spks)| {
    //         let mut first = true;
    //         (
    //             k,
    //             spks.inspect(move |(spk_i, _)| {
    //                 if first {
    //                     first = false;
    //                     print!("\nScanning keychain [{:?}]:", k);
    //                 }
    //                 print!(" {}", spk_i);
    //                 let _ = std::io::stdout().flush();
    //             }),
    //         )
    //     })
    //     .collect();
    // let update = client
    //     .scan(
    //         checkpoints,
    //         spks,
    //         core::iter::empty(),
    //         core::iter::empty(),
    //         STOP_GAP,
    //         PARALLEL_REQUESTS,
    //     )
    //     .expect("failed to scan");

    // wallet.apply_update(update).expect("failed to apply update");
    // wallet.commit().expect("failed to commit update");

    // if balance.total() < SEND_AMOUNT {
    //     println!(
    //         "Please send at least {} sats to the receiving address",
    //         SEND_AMOUNT
    //     );
    //     std::process::exit(0);
    // }

    // let faucet_address = Address::from_str("mkHS9ne12qx9pS9VojpwU5xtRd4T7X7ZUt").expect("Failed to parse address");

    // let mut tx_builder = wallet.build_tx();
    // tx_builder
    //     .add_recipient(faucet_address.script_pubkey(), SEND_AMOUNT)
    //     .enable_rbf();

    // let (mut psbt, _) = tx_builder.finish().expect("Failed to build transaction");
    // let finalized = wallet.sign(&mut psbt, SignOptions::default()).expect("Failed to sign transaction");
    // assert!(finalized);

    // let tx = psbt.extract_tx();
    // client.broadcast(&tx).expect("Failed to broadcast transaction");
    // println!("Tx broadcasted! Txid: {}", tx.txid());
}
