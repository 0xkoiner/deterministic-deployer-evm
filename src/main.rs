mod client;
mod types;
mod utils;

use crate::client::wallet_client::WalletClient;
use utils::init_rpc::{config, get_rpc};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let config = config();
    let eth_rpc = match get_rpc(&config, "mainnet", "ethereum").await {
        Ok(url) => url.to_string(),
        Err(e) => format!("Error: {e}"),
    };
    let sepolia_rpc = match get_rpc(&config, "testnet", "sepolia").await {
        Ok(url) => url.to_string(),
        Err(e) => format!("Error: {e}"),
    };

    info!("{eth_rpc}");
    info!("{sepolia_rpc}");
    // println!("{eth_rpc}");
    // println!("{sepolia_rpc}");

    let network_error = match get_rpc(&config, "error", "sepolia").await {
        Ok(url) => url.to_string(),
        Err(e) => format!("Error: {e}"),
    };
    error!("{network_error}");
    // println!("{network_error}");

    let chain_error = match get_rpc(&config, "mainnet", "error").await {
        Ok(url) => url.to_string(),
        Err(e) => format!("Error: {e}"),
    };
    error!("{chain_error}");
    // println!("{chain_error}");

    let wallet = WalletClient::from_env().expect("Failed to create wallet");
    info!("Wallet address: {}", wallet.address());
    // println!("Wallet address: {}", wallet.address());

    let wallet2 = WalletClient::from_env().expect("Failed to create wallet");
    let eth_rpc2 = get_rpc(&config, "mainnet", "ethereum")
        .await
        .expect("Failed to get RPC");

    info!("Wallet address: {}", wallet2.address());
    info!("Wallet Rpc: {}", eth_rpc2);

    // println!("Wallet address: {}", wallet2.address());
    // println!("Wallet Rpc: {}", eth_rpc2);
}
