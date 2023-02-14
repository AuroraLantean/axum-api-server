use ethers::contract::Contract;
use ethers::prelude::{BlockNumber, ContractFactory, LocalWallet, Signer, SignerMiddleware, U256};
use ethers::utils::Ganache;
use ethers_providers::{Middleware, Provider};
use ethers_solc::Artifact;
use ethers_solc::{ConfigurableArtifacts, Project, ProjectCompileOutput, ProjectPathsConfig};
use eyre::Result;
use eyre::{eyre, ContextCompat};
use hex::ToHex;
use std::path::PathBuf;
use std::time::Duration;

pub type SignerDeployedContract<T> = Contract<SignerMiddleware<Provider<T>, LocalWallet>>;

pub async fn compile_deploy_contract() -> Result<()> {
    println!("------------------== compile_deploy_contract");
    // Spawn a ganache instance
    let mnemonic = dotenvy::var("MNEMONIC").expect("MNEMONIC not found");
    println!("MNEMONIC is valid: {}", &mnemonic);

    let ganache = Ganache::new().mnemonic(mnemonic).spawn();
    println!("HTTP Endpoint: {}", ganache.endpoint());

    // Get the first wallet managed by ganache
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let addr0 = wallet.address();
    println!("Wallet[0] address: {}", addr0.encode_hex::<String>());

    let ethereum_endpoint = ganache.endpoint();
    //let ethereum_endpoint = "127.0.0.1:8545".to_owned();
    // A provider is an Ethereum JsonRPC client
    let provider = Provider::try_from(ethereum_endpoint)?.interval(Duration::from_millis(10));
    let chain_id = provider.get_chainid().await?.as_u64();
    println!("Ganache started with chain_id {chain_id}");

    // Compile solidity project
    // this root_str MUST be a "folderName/" next to src folder! AND it is included in the "contract_path" below!
    let project = compile_contracts("contracts/").await?;
    println!("compile_contracts succeeded");
    // Print compiled project information
    print_project(project.clone()).await?;
    println!("print_project succeeded");

    let balance = provider.get_balance(wallet.address(), None).await?;

    println!(
        "addr0 {} balance: {}",
        wallet.address().encode_hex::<String>(),
        balance
    );

    let contract_name = "BUSDImplementation";
    let contract_path = "BUSDImplementation.sol";
    // Find the contract to be deployed
    let contract = project
        .find(contract_path, contract_name)
        .context("Contract not found")?
        .clone();
    println!("contract is found");
    // make a transaction to include code for deploying the contract
    // Get ABI and contract byte, these are required for contract deployment
    let (abi_option, bytecode, _) = contract.into_parts();
    println!("abi, bytecode parsed succeeded");
    let contract = abi_option.context("Missing abi from contract")?;
    println!("contract extracted");
    let bytecode = bytecode.context("Missing bytecode from contract")?;

    // Make signer client
    let signer = wallet.with_chain_id(chain_id);
    println!("signer found from chain_id");
    let client = SignerMiddleware::new(provider.clone(), signer).into();
    println!("client found");
    // Deploy contract
    let factory = ContractFactory::new(contract.clone(), bytecode, client);
    println!("factory found");
    // Our contract don't need any constructor arguments, so we can use an empty tuple
    let mut deployer = factory.deploy(())?;
    println!("deployer found from factory");

    let block = provider
        .clone()
        .get_block(BlockNumber::Latest)
        .await?
        .context("Failed to get latest block")?;
    println!("block found");
    // Set a reasonable gas price to prevent our contract from being rejected by EVM
    let gas_price = block
        .next_block_base_fee()
        .context("Failed to get the next block base fee")?;
    println!("gas_price found");
    deployer.tx.set_gas_price::<U256>(gas_price);
    println!("set_gas_price() succeeded");
    // We can also manually set the gas limit
    // let gas_limit = block.gas_limit;
    // deployer.tx.set_gas::<U256>(gas_limit);

    // Send deployment transaction
    let contract = deployer.clone().legacy().send().await?;
    println!("deployment succeeded");
    println!(
        "{} contract address: {}",
        contract_name,
        contract.address().encode_hex::<String>()
    );

    Ok(())
}

pub async fn compile_contracts(
    root_str: &str,
) -> Result<ProjectCompileOutput<ConfigurableArtifacts>> {
    println!("--------------== compile_contracts");
    // Make path from string and check if the path exists
    let root = PathBuf::from(root_str);
    if !root.exists() {
        println!("root does not exist. {}", root_str);
        return Err(eyre!("Project root {root:?} does not exists!"));
    }
    println!("root building succeeded");
    // Configure `root` as our project root
    let paths = ProjectPathsConfig::builder()
        .root(&root)
        .sources(&root)
        .build()?;
    println!("paths building succeeded");

    // Make a solc ProjectBuilder instance for compilation
    let project = Project::builder()
        .paths(paths)
        .set_auto_detect(true) // auto detect solc version from solidity source code
        .no_artifacts()
        .build()?;
    println!("project building succeeded");

    // Install Solc !!!
    // Compile project
    let output = project.compile()?;
    println!("project.compile() succeeded");

    // Check for compilation errors
    if output.has_compiler_errors() {
        println!("output failed");
        Err(eyre!(
            "Compiling solidity project failed: {:?}",
            output.output().errors
        ))
    } else {
        println!("output succeeded");
        Ok(output.clone())
    }
}

pub async fn print_project(project: ProjectCompileOutput<ConfigurableArtifacts>) -> Result<()> {
    println!("--------------== print_project");
    let artifacts = project.into_artifacts();
    for (id, artifact) in artifacts {
        let name = id.name;
        let abi = artifact.abi.context("No ABI found for artificat {name}")?;

        println!("{}", "=".repeat(80));
        println!("CONTRACT: {:?}", name);

        let abi = &abi.abi;
        let functions = abi.functions();
        let functions = functions.cloned();
        let constructor = abi.constructor();

        if let Some(constructor) = constructor {
            let args = &constructor.inputs;
            println!("CONSTRUCTOR args: {args:?}");
        }

        for func in functions {
            let name = &func.name;
            let params = &func.inputs;
            println!("FUNCTION  {name} {params:?}");
        }
    }
    Ok(())
}
