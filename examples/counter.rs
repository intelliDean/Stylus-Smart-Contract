//! Example on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
};
use eyre::Error;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();

    let rpc_url = env::var("RPC_URL")?;
    let priv_key_path = env::var("PRIV_KEY_PATH")?;

    let contract_address: Address = env::var("STYLUS_CONTRACT_ADDRESS")?
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid contract address"))
        .unwrap();

    let provider = Provider::<Http>::try_from(&rpc_url)?.interval(Duration::from_millis(1000));
    let chain_id = provider.get_chainid().await?.as_u64();

    let wallet = priv_key_path
        .parse::<LocalWallet>()?
        .with_chain_id(chain_id);
    let eth_client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

    abigen!(
        Degen,
        r"[
            function addressZeroCheck() external view
            function regCheck() external
            function onlyOwner() external view
            function playerRegister(string calldata username) external
            function playGame() external
            function distributeRewardToPlayers() external
            function playerP2PTransfer(address recipient, uint256 value) external
            function suspendPlayer(address _address) external
            function reinstatePlayer(address _address) external
            function playerCheckBalance() external view returns (uint256)
            function playerBurnToken(uint256 value) external
            function addGameProp(string calldata name, uint256 value) external
            function playerBuysFromGameStore(bytes32 prop_id) external
        ]"
    );

    let degen = Degen::new(contract_address, eth_client);
    println!("Contract: {degen:?}");

    let player_address: Address = "0xc6fB3fe7C22220862A1b403e5FECE8F13bcB61CE"
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid contract address"))
        .unwrap();

    // //todo: player registers => PLAYER
    // degen.player_register("Dean".to_string()).send().await?;
    //
    // println!("Player registers");

    // //TODO: Player plays game => PLAYER
    // degen.play_game().send().await?;
    // println!("Player plays game");
    //
    // //TODO: Admin distributes reward => ADMIN
    degen.distribute_reward_to_players().send().await?;
    println!("Admin distributes rewards");
    //
    // // //todo: Player checks balance
    // let balance = degen.player_check_balance().call().await?;
    // println!("Player Balance = {:?}", balance);

    Ok(())
}
