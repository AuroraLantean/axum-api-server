use core::fmt;
use ethers::{
    prelude::*,
    types::{BlockNumber, H256},
    utils::{format_units, parse_ether},
};
use ethers_providers::{Authorization, Http};
use eyre::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use std::sync::Arc;

async fn _create_instance(rpc_url: &str) -> eyre::Result<()> {
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
async fn _share_providers_across_tasks(rpc_url: &str) -> eyre::Result<()> {
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
#[derive(Debug)]
pub struct Env {
    pub mnemonic: String,
    pub network: String,
    pub rpc_url: String,
    pub pvkey0: String,
    pub addr0: String,
    pub addr1: String,
    pub addr2: String,
    pub addr3: String,
    pub erc20_token_addr: String,
}
pub fn get_env_parameters() -> Result<Env, String> {
    // MNEMONIC has to be enclosed in double quotes for .env to work!!!
    let mnemonic = dotenvy::var("MNEMONIC").map_err(|_e| "MNEMONIC not found".to_owned())?;
    println!("MNEMONIC is valid: {}", &mnemonic);

    let network = dotenvy::var("NETWORK").map_err(|_e| "NETWORK not found".to_owned())?;
    println!("NETWORK is valid: {}", &network);

    let infura_key = dotenvy::var("INFURA_KEY").map_err(|_e| "INFURA_KEY not found".to_owned())?;
    println!("INFURA_KEY is valid: {}", &infura_key);

    let rpc_url = [
        "https://",
        network.as_str(),
        ".infura.io/v3/",
        infura_key.as_str(),
    ]
    .join("");
    println!("rpc_url: {}", &rpc_url);

    let addr0 = dotenvy::var("ETH_ADDR0").map_err(|_e| "ETH_ADDR0 not found".to_owned())?;
    println!("ETH_ADDR0 is valid: {}", &addr0);

    let pvkey0 = dotenvy::var("PRIVATE_KEY0").map_err(|_e| "PRIVATE_KEY0 not found".to_owned())?;
    println!("PRIVATE_KEY0 is valid: {}", &pvkey0);

    let addr1 = dotenvy::var("ETH_ADDR1").map_err(|_e| "ETH_ADDR1 not found".to_owned())?;
    println!("ETH_ADDR1 is valid: {}", &addr1);

    let addr2 = dotenvy::var("ETH_ADDR2").map_err(|_e| "ETH_ADDR2 not found".to_owned())?;
    println!("ETH_ADDR2 is valid: {}", &addr2);

    let addr3 = dotenvy::var("ETH_ADDR3").map_err(|_e| "ETH_ADDR3 not found".to_owned())?;
    println!("ETH_ADDR3 is valid: {}", &addr3);

    let erc20_token_addr =
        dotenvy::var("ETH_ERC20TOKEN").map_err(|_e| "ETH_ERC20TOKEN not found".to_owned())?;
    println!("ETH_ERC20TOKEN is valid: {}", &erc20_token_addr);

    Ok(Env {
        mnemonic,
        network,
        rpc_url,
        pvkey0,
        addr0,
        addr1,
        addr2,
        addr3,
        erc20_token_addr,
    })
}
pub async fn get_write_provider(
    rpc_url: &str,
    private_key: &str,
) -> Result<
    Arc<
        ethers::middleware::SignerMiddleware<
            ethers_providers::Provider<Http>,
            Wallet<ethers::core::k256::ecdsa::SigningKey>,
        >,
    >,
    String,
> {
    let provider = Arc::new({
        println!("get_write_provider 0");
        let provider =
            Provider::<Http>::try_from(rpc_url).map_err(|_e| "provider error1".to_owned())?;
        let chain_id = provider
            .get_chainid()
            .await
            .map_err(|_e| "get_chainid error".to_owned())?;
        println!("get_write_provider 1");
        // this wallet's private key
        let wallet = private_key
            .parse::<LocalWallet>()
            .map_err(|_e| "private_key.parse error".to_owned())?
            .with_chain_id(chain_id.as_u64());
        println!("get_write_provider 2");

        SignerMiddleware::new(provider, wallet)
    });
    Ok(provider)
}
pub async fn ethereum_live_write(amount_in_eth: f64) -> Result<(String, String)> {
    println!("ethereum_live_write 0");
    let env = get_env_parameters().expect("env error");
    let provider = get_write_provider(&env.rpc_url, &env.pvkey0)
        .await
        .expect("provider error");
    println!("ethereum_live_write 1");

    let address = env.erc20_token_addr.parse::<Address>()?;
    let erc20token = ERC20Token::new(address, Arc::clone(&provider));
    println!("ethereum_live_write 2");
    let addr0 = env.addr0.parse::<Address>()?;
    let addr1 = env.addr1.parse::<Address>()?;
    let balance0 = erc20token.balance_of(addr0).call().await?;
    println!("balance0: ({balance0})");
    let balance1 = erc20token.balance_of(addr1).call().await?;
    println!("balance1: ({balance1})");
    //let amount_in_eth = 17u64;
    let receipt = erc20token
        .transfer(addr1, parse_ether(amount_in_eth)?)
        .send()
        .await?
        .await?
        .expect("no receipt found");
    println!("receipt made");
    println!("{receipt:?}");
    let receipt_txn_hash = receipt.transaction_hash;
    println!("receipt_txn_hash: {}", receipt_txn_hash);
    let txn_hash = NewH256(receipt_txn_hash);
    println!("txn_hash: {}", txn_hash);

    let tx = provider.get_transaction(receipt_txn_hash).await?;
    println!("tx confirmed: {}\n", serde_json::to_string(&tx)?);
    println!("receipt: {}", serde_json::to_string(&receipt)?);

    let balance0 = erc20token.balance_of(addr0).call().await?;
    println!("balance0: ({balance0})");
    let balance1 = erc20token.balance_of(addr1).call().await?;
    println!("balance1: ({balance1})");
    let bal1 = format_units(balance1, "ether").unwrap();
    Ok((txn_hash.to_string(), bal1))
}

struct NewH256(H256);
impl fmt::Display for NewH256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x")?;
        for i in &self.0[..] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

pub fn get_read_provider(rpc_url: &str) -> Result<Arc<Provider<Http>>, String> {
    println!("get_read_provider 0");
    let prov = Provider::<Http>::try_from(rpc_url).map_err(|_e| "provider error1".to_owned())?;
    println!("get_read_provider 1");
    let provider = Arc::new(prov);
    Ok(provider)
}
pub async fn ethereum_live_read() -> Result<(String, String)> {
    //create_instance(&rpc_url).await?;
    //println!("create_instance succeeded");
    //share_providers_across_tasks().await?;
    //println!("share_providers_across_tasks succeeded");
    println!("ethereum_live_read 0");
    let env = get_env_parameters().expect("env error");
    //.map_err(|_e| "env error".to_owned())?;
    println!("ethereum_live_read 1");
    let provider = get_read_provider(&env.rpc_url).expect("provider error");
    //let client = Provider::<Http>::try_from(env.rpc_url)?;
    //let client = Arc::new(client);
    println!("ethereum_live_read 2");

    let last_block = provider
        .get_block(BlockNumber::Latest)
        .await?
        .unwrap()
        .number
        .unwrap();
    println!("last_block: {last_block}");

    let address = env.erc20_token_addr.parse::<Address>()?;
    let erc20token = ERC20Token::new(address, Arc::clone(&provider));
    println!("ethereum_live_read 3");

    let ctrt_name = erc20token.name().call().await?;
    println!("ctrt_name: ({ctrt_name})");
    let total_supply = erc20token.total_supply().call().await?;
    println!("total_supply: ({total_supply})");

    let addr0 = env.addr0.parse::<Address>()?;
    let addr1 = env.addr1.parse::<Address>()?;
    let balance0 = erc20token.balance_of(addr0).call().await?;
    println!("balance0: ({balance0})");
    let balance1 = erc20token.balance_of(addr1).call().await?;
    println!("balance1: ({balance1})");
    let bal0 = format_units(balance0, "ether").unwrap();
    let bal1 = format_units(balance1, "ether").unwrap();
    dbg!(&bal0);
    dbg!(&bal1);
    /*
    let (reserve0, reserve1, _timestamp) = ...
    let mid_price = f64::powi(10.0, 18 - 6) * reserve1 as f64 / reserve0 as f64;
    println!("ETH/USDT price: {mid_price:.2}");
    */

    Ok((bal0, bal1))
}

pub async fn ethereum_send_ether(
    _to_addr_str: String,
    amount_in_eth: f64,
) -> Result<(String, String)> {
    println!("----------== ethereum_send_ether");
    println!("ethereum_send_ether 0");
    let env = get_env_parameters().expect("env error");
    let provider = get_write_provider(&env.rpc_url, &env.pvkey0)
        .await
        .expect("provider error");
    println!("ethereum_send_ether 1");

    //let accounts = provider.get_accounts().await?;
    let from = env.addr0.parse::<Address>()?;
    let to_addr = env.addr1.parse::<Address>()?;
    //let to_addr = to_addr_str.parse::<Address>()?;

    let amount_in_u256 = parse_ether(amount_in_eth)?;
    println!("ethereum_send_ether 2: inputs are valid");
    // craft the tx
    let tx = TransactionRequest::new()
        .to(to_addr)
        .value(amount_in_u256)
        .from(from);
    println!("ethereum_send_ether 3: tx made");

    let balance_before = provider.get_balance(from, None).await?;
    let nonce1 = provider.get_transaction_count(from, None).await?;
    dbg!(balance_before, nonce1);

    // broadcast it via the eth_sendTransaction API
    let pending_tx = provider.send_transaction(tx, None).await?.await?;
    println!("ethereum_send_ether 4: tx pending");
    println!("{}", serde_json::to_string(&pending_tx)?);

    //pending_tx.confirmations(3).await?;
    let receipt = pending_tx.ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
    println!("ethereum_send_ether 5: receipt made");
    println!("{receipt:?}");
    let receipt_txn_hash = receipt.transaction_hash;
    println!("receipt_txn_hash: {}", receipt_txn_hash);
    let txn_hash = NewH256(receipt_txn_hash);
    println!("txn_hash: {}", txn_hash);

    let tx = provider.get_transaction(receipt_txn_hash).await?;
    println!("ethereum_send_ether 6: tx confirmed");
    println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    println!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    let nonce2 = provider.get_transaction_count(from, None).await?;
    println!("check below: nonce1 < nonce2");
    dbg!(nonce1, nonce2);

    let balance_after = provider.get_balance(from, None).await?;
    println!("check below: balance_after < balance_before");
    dbg!(balance_before, balance_after);

    let bal1 = format_units(balance_after, "ether").unwrap();
    Ok((txn_hash.to_string(), bal1))
}
/*
   let base: U256 = U256::from(10).pow(ETH_DECIMALS.into());
   let value: U256 = amount.mul(price_usd).div(base);
   let f: String = format_units(value, USD_PRICE_DECIMALS)?;
   Ok(f.parse::<f64>()?)
*/
