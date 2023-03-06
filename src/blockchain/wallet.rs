use secp256k1::{PublicKey, SecretKey, rand::{rngs, SeedableRng}};
//use anyhow::Result;//error handling
use ethers::{
  core::rand,
  signers::{coins_bip39::English, MnemonicBuilder},
};
use eyre::Result;
// use web3::Web3;
// use web3::transports::Http;
// use web3::types::{TransactionParameters, U256, H256, H160};

pub fn make_keypair() -> Result<(SecretKey, PublicKey)>{
    println!("----------== make_keypair");
    let secp = secp256k1::Secp256k1::new();
    let mut rng = rngs::StdRng::seed_from_u64(6);
    Ok(secp.generate_keypair(&mut rng))
}
pub fn make_new_mnemonic() -> Result<()>{
    println!("----------== make_new_mnemonic");
    // Generate a random wallet (24 word phrase) at custom derivation path
    let mut rng = rand::thread_rng();
    let wallet_addr = MnemonicBuilder::<English>::default()
        .word_count(24)
        .derivation_path("m/44'/60'/0'/2/1")?
        // Optionally add this if you want the generated mnemonic to be written
        // to a file
        // .write_to(path)
        .build_random(&mut rng)?;
    dbg!(&wallet_addr);
    Ok(())
}
pub fn get_address_from_mnemonic() -> Result<()>{
  println!("----------== get_address_from_mnemonic");
  let phrase = "work man father plunge mystery proud hollow address reunion sauce theory bonus";
  let index = 0u32;
  let password = "TREZOR123";

  // Access mnemonic phrase with password
  // Child key at derivation path: m/44'/60'/0'/0/{index}
  let wallet = MnemonicBuilder::<English>::default()
      .phrase(phrase)
      .index(index)?
      // Use this if your mnemonic is encrypted
      .password(password)
      .build()?;
  dbg!(&wallet);
  Ok(())
}
/*
pub fn establish_web3_connection(url: &str) -> Result<Web3<Http>>{
  println!("----------== make_keypair");
  let transport = web3::transports::Http::new(url)?;
    Ok(web3::Web3::new(transport))
}

pub fn create_txn_object(to: H160, value: usize)-> Result<TransactionParameters>{
  Ok(TransactionParameters {
        to: Some(to),
        //todo: check value
        value: U256::exp10(value), //0.1 eth
        ..Default::default()
    })
}

pub async fn sign_and_send(web3: Web3<Http>, tx_object: TransactionParameters, seckey: SecretKey) -> Result<H256>{
    println!("----------== make_keypair");
    let signed = web3.accounts().sign_transaction(tx_object, &seckey).await?;
    Ok(web3.eth().send_raw_transaction(signed.raw_transaction).await?)
}
 */
