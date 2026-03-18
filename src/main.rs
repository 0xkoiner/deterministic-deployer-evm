mod types;                                                                                                                           
mod utils;

use crate::types::types::RpcConfig;
use utils::init_rpc::{load_config, get_rpc};

#[tokio::main]                                                                                                                       
async fn main() {
    let config:RpcConfig = load_config().await.unwrap();
    let eth_rpc: String = get_rpc(&config, "mainnet", "ethereum").unwrap();
    let sepolia_rpc: String = get_rpc(&config, "testnet", "sepolia").unwrap();
    
    println!("{eth_rpc}");
    println!("{sepolia_rpc}");
    
    // let network_error: String = get_rpc(&config, "error", "sepolia").unwrap();
    // println!("{network_error}");
    // let chain_error: String = get_rpc(&config, "mainnet", "error").unwrap();
    // println!("{chain_error}");
}  
