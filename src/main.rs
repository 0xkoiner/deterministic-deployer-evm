mod types;
mod utils;

use crate::types::config::RpcConfig;
use utils::init_rpc::{get_rpc, load_config};

#[tokio::main]
async fn main() {
    let config: RpcConfig = load_config().expect("Failed to load RPC config");
    let eth_rpc = match get_rpc(&config, "mainnet", "ethereum").await {
        Ok(url) => url,
        Err(e) => format!("Error: {e}"),
    };
    let sepolia_rpc = match get_rpc(&config, "testnet", "sepolia").await {
        Ok(url) => url,
        Err(e) => format!("Error: {e}"),
    };

    println!("{eth_rpc}");
    println!("{sepolia_rpc}");

    let network_error = match get_rpc(&config, "error", "sepolia").await {
        Ok(url) => url,
        Err(e) => format!("Error: {e}"),
    };
    println!("{network_error}");

    let chain_error = match get_rpc(&config, "mainnet", "error").await {
        Ok(url) => url,
        Err(e) => format!("Error: {e}"),
    };
    println!("{chain_error}");
}
