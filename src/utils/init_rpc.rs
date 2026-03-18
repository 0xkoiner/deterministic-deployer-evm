/**
 *  1. Read a TOML file from disk
 *  2. Parse it into a Rust data structure
 *  3. Look up the value by network (mainnet/testnet) → then by chain name
 *  4. Return the RPC URL string"];
 */

use tokio::fs;                                                                                                                       
use crate::types::types::RpcConfig;
use crate::types::constants::RPC_TOML_PATH;                                                                                          
                                                                                                                                    
pub async fn load_config() -> Result<RpcConfig, Box<dyn std::error::Error>> {                                                        
    let toml = fs::read_to_string(RPC_TOML_PATH).await?;                                                                             
    let config: RpcConfig = toml::from_str(&toml)?;                                                                                  
    Ok(config)
}                                                                                                                                    
                
pub fn get_rpc(config: &RpcConfig, network: &str, chain: &str) -> Result<String, Box<dyn std::error::Error>> {                       
    let table = match network {
        "mainnet" => &config.mainnet,                                                                                                
        "testnet" => &config.testnet,
        _ => return Err("Unknown network. Use mainnet or testnet".into()),                                                           
    };                                                                                                                               
                                                                                                                                    
    match table.get(chain) {                                                                                                         
        Some(url) => Ok(url.clone()),
        None => Err(format!("Chain {chain} not found in {network}").into()),
    }                                                                                                                                
}