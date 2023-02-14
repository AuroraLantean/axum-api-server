use ethers::{
    prelude::*,
    types::{BlockNumber, H256},
    utils::parse_ether,
};
use ethers_providers::{Authorization, Http};
use eyre::{ContextCompat, Result};
use hex::ToHex;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::Arc;

async fn create_instance(rpc_url: &str) -> eyre::Result<()> {
    // An Http provider can be created from an http(s) URI.
    // In case of https you must add the "rustls" or "openssl" feature
    // to the ethers library dependency in `Cargo.toml`.
    let _provider = Provider::<Http>::try_from(rpc_url)?;

    // Instantiate with auth to append basic authorization headers across requests
    let url = reqwest::Url::parse(rpc_url)?;
    let auth = Authorization::basic("username", "password");
    let _provider = Http::new_with_auth(url, auth)?;

    // Instantiate from custom Http Client if you need
    // finer control over the Http client configuration
    // (TLS, Proxy, Cookies, Headers, etc.)
    let url = reqwest::Url::parse(rpc_url)?;

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static("Bearer my token"));
    headers.insert("X-MY-HEADERS", HeaderValue::from_static("Some value"));

    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .proxy(reqwest::Proxy::all("http://proxy.example.com:8080")?)
        .build()?;

    let _provider = Http::new_with_client(url, http_client);

    Ok(())
}

/// Providers can be easily shared across tasks using `Arc` smart pointers
async fn share_providers_across_tasks(rpc_url: &str) -> eyre::Result<()> {
    let provider: Provider<Http> = Provider::<Http>::try_from(rpc_url)?;

    let client_1 = Arc::new(provider);
    let client_2 = Arc::clone(&client_1);

    let handle1 = tokio::spawn(async move {
        client_1
            .get_block(BlockNumber::Latest)
            .await
            .unwrap_or(None)
    });

    let handle2 = tokio::spawn(async move {
        client_2
            .get_block(BlockNumber::Latest)
            .await
            .unwrap_or(None)
    });

    let block1: Option<Block<H256>> = handle1.await?;
    let block2: Option<Block<H256>> = handle2.await?;

    println!("{block1:?} {block2:?}");

    Ok(())
}

// Generate the type-safe contract bindings by providing the ABI definition in human readable format
abigen!(ERC20Token, "ERC20Token.json");
/**
   r#"[
     function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
 ]"#
*/

pub async fn ethereum_live_txn() -> Result<()> {
    // Spawn a ganache instance ... MNEMONIC has to be enclosed in double quotes for .env to work!!!
    let mnemonic = dotenvy::var("MNEMONIC").expect("MNEMONIC not found");
    println!("MNEMONIC is valid: {}", &mnemonic);

    let network = dotenvy::var("NETWORK").expect("NETWORK not found");
    println!("NETWORK is valid: {}", &network);

    let infura_key = dotenvy::var("INFURA_KEY").expect("INFURA_KEY not found");
    println!("INFURA_KEY is valid: {}", &infura_key);

    let rpc_url = [
        "https://",
        network.as_str(),
        ".infura.io/v3/",
        infura_key.as_str(),
    ]
    .join("");
    println!("rpc_url: {}", rpc_url);

    let addr0 = dotenvy::var("ETH_ADDR0").expect("ETH_ADDR0 not found");
    println!("ETH_ADDR0 is valid: {}", &addr0);

    let addr1 = dotenvy::var("ETH_ADDR1").expect("ETH_ADDR1 not found");
    println!("ETH_ADDR1 is valid: {}", &addr1);

    let erc20_token_addr = dotenvy::var("ETH_ERC20TOKEN").expect("ETH_ERC20TOKEN not found");
    println!("ETH_ERC20TOKEN is valid: {}", &erc20_token_addr);

    //create_instance(&rpc_url).await?;
    //println!("create_instance succeeded");
    //share_providers_across_tasks().await?;
    //println!("share_providers_across_tasks succeeded");

    let client = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(client);

    let last_block = client
        .get_block(BlockNumber::Latest)
        .await?
        .unwrap()
        .number
        .unwrap();
    println!("last_block: {last_block}");

    let address = erc20_token_addr.parse::<Address>()?;
    let erc20token = ERC20Token::new(address, Arc::clone(&client));

    let ctrt_name = erc20token.name().call().await?;
    println!("ctrt_name: ({ctrt_name})");
    let total_supply = erc20token.total_supply().call().await?;
    println!("total_supply: ({total_supply})");

    let addr0 = addr0.parse::<Address>()?;
    let addr1 = addr1.parse::<Address>()?;
    let balance0 = erc20token.balance_of(addr0).call().await?;
    println!("balance0: ({balance0})");
    let balance1 = erc20token.balance_of(addr1).call().await?;
    println!("balance1: ({balance1})");

    let amount_in_eth = 17;
    let receipt = erc20token
        .transfer(addr1, parse_ether(amount_in_eth)?)
        .send()
        .await?
        .await?
        .expect("no receipt found");
    println!("receipt made");
    println!("{receipt:?}");

    let tx = client.get_transaction(receipt.transaction_hash).await?;
    println!("tx hash confirmed");
    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    let balance0 = erc20token.balance_of(addr0).call().await?;
    println!("balance0: ({balance0})");
    let balance1 = erc20token.balance_of(addr1).call().await?;
    println!("balance1: ({balance1})");
    /*
    let (reserve0, reserve1, _timestamp) = ...
    let mid_price = f64::powi(10.0, 18 - 6) * reserve1 as f64 / reserve0 as f64;
    println!("ETH/USDT price: {mid_price:.2}");
    */

    Ok(())
}

pub async fn ethereum_send_token(
    erc20token: &Arc<Provider<Http>>,
    to_addr: H160,
    amount_in_eth: u64,
    //amount: U256,
) -> Result<()> {
    println!("----------== ethereum_send_token");

    Ok(())
}

pub async fn ethereum_send_ETH(
    client: &Arc<Provider<Http>>,
    to_addr: H160,
    amount_in_eth: u64,
    //amount: U256,
) -> Result<()> {
    println!("----------== ethereum_send_ETH");
    let tx = TransactionRequest::new()
        .to(to_addr)
        .value(parse_ether(amount_in_eth)?);
    println!("tx made");
    // send it!
    let pending_tx = client.send_transaction(tx, None).await?;
    println!("pending_tx made & tx sent");

    //pending_tx.confirmations(3).await?;
    let receipt = pending_tx
        .await?
        .ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
    println!("receipt made");

    let tx = client.get_transaction(receipt.transaction_hash).await?;
    println!("tx hash confirmed");
    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);
    Ok(())
}
/*
   let base: U256 = U256::from(10).pow(ETH_DECIMALS.into());
   let value: U256 = amount.mul(price_usd).div(base);
   let f: String = format_units(value, USD_PRICE_DECIMALS)?;
   Ok(f.parse::<f64>()?)
*/
