mod client;
mod types;
mod utils;

use crate::client::wallet_client::WalletClient;
use crate::types::config::RpcConfig;
use utils::init_rpc::{get_rpc, load_config};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let config: RpcConfig = load_config().expect("Failed to load RPC config");
    let eth_rpc = match get_rpc(&config, "mainnet", "ethereum").await {
        Ok(url) => url.to_string(),
        Err(e) => format!("Error: {e}"),
    };
    let sepolia_rpc = match get_rpc(&config, "testnet", "sepolia").await {
        Ok(url) => url.to_string(),
        Err(e) => format!("Error: {e}"),
    };

    println!("{eth_rpc}");
    println!("{sepolia_rpc}");

    let network_error = match get_rpc(&config, "error", "sepolia").await {
        Ok(url) => url.to_string(),
        Err(e) => format!("Error: {e}"),
    };
    println!("{network_error}");

    let chain_error = match get_rpc(&config, "mainnet", "error").await {
        Ok(url) => url.to_string(),
        Err(e) => format!("Error: {e}"),
    };
    println!("{chain_error}");

    let wallet = WalletClient::from_env().expect("Failed to create wallet");
    println!("Wallet address: {}", wallet.address());

    let wallet2 = WalletClient::from_env().expect("Failed to create wallet");
    let eth_rpc2 = get_rpc(&config, "mainnet", "ethereum")
        .await
        .expect("Failed to get RPC");

    println!("Wallet address: {}", wallet2.address());
    println!("Wallet Rpc: {}", eth_rpc2);
}
