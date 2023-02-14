use std::time::Duration;

use ethers::{
    prelude::{Address, LocalWallet, Middleware, Provider, Signer, TransactionRequest, U256},
    utils::Ganache,
};
use eyre::{ContextCompat, Result};
use hex::ToHex;

pub async fn ethereum_local_txn() -> Result<()> {
    // Spawn a ganache instance ... MNEMONIC has to be enclosed in double quotes for .env to work!!!
    let mnemonic = dotenvy::var("MNEMONIC").expect("MNEMONIC not found");
    println!("MNEMONIC is valid: {}", &mnemonic);

    let addr1 = dotenvy::var("ETH_ADDR1").expect("ETH_ADDR1 not found");
    println!("ETH_ADDR1 is valid: {}", &addr1);

    // Ganache-cli has to be installed globally first!
    // npm install ganache --global
    let ganache = Ganache::new().mnemonic(mnemonic).spawn();
    println!("Ganache is successful");
    println!("HTTP Endpoint: {}", ganache.endpoint());
    //HTTP Endpoint: http://localhost:38265

    // Get the first wallet managed by ganache
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let addr0 = wallet.address();
    println!("Wallet[0] address: {}", addr0.encode_hex::<String>());

    // A provider is an Ethereum JsonRPC client
    let ethereum_endpoint = ganache.endpoint();
    //let ethereum_endpoint = "127.0.0.1:8545".to_owned();
    let provider = Provider::try_from(ethereum_endpoint)?.interval(Duration::from_millis(10));
    let chain_id = provider.get_chainid().await?.as_u64();
    println!("Ganache started with chain_id {chain_id}");

    // Query the balance of our account
    let first_balance = provider.get_balance(addr0, None).await?;
    println!("Wallet first address balance: {}", first_balance);

    // Query the blance of some random account
    let other_address_hex = addr1;
    let other_address = other_address_hex.parse::<Address>()?;
    let other_balance = provider.get_balance(other_address, None).await?;
    println!("Balance of {}: {}", other_address_hex, other_balance);

    // Create a transaction to transfer 1000 wei to `other_address`
    let tx = TransactionRequest::pay(other_address, U256::from(1000u64)).from(addr0);
    // Send the transaction and wait for receipt
    let receipt = provider
        .send_transaction(tx, None)
        .await?
        .log_msg("Pending transfer")
        .confirmations(1) // number of confirmations required
        .await?
        .context("Missing receipt")?;

    println!(
        "tx mined in block {}",
        receipt.block_number.context("Can not get block number")?
    );
    println!(
        "Balance of {}: {}",
        other_address_hex,
        provider.get_balance(other_address, None).await?
    );

    Ok(())
}
