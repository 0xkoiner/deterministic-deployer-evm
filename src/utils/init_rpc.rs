// 1. Read a TOML file from disk
// 2. Parse it into a Rust data structure
// 3. Look up the value by network (mainnet/testnet) → then by chain name
// 4. Return the RPC URL string
use crate::types::config::RpcConfig;
use crate::types::constants::RPC_TOML;
use crate::types::errors::RpcError;
use std::collections::HashMap;

fn _validate_network<'a>(
    config: &'a RpcConfig,
    network: &str,
) -> Result<&'a HashMap<String, String>, RpcError> {
    match network {
        "mainnet" => Ok(&config.mainnet),
        "testnet" => Ok(&config.testnet),
        _ => Err(RpcError::UnknownNetwork(network.to_string())),
    }
}

fn _validate_chain<'a>(
    table: &'a HashMap<String, String>,
    chain: &str,
    network: &str,
) -> Result<&'a str, RpcError> {
    table
        .get(chain)
        .map(|s| s.as_str())
        .ok_or_else(|| RpcError::ChainNotFound(chain.to_string(), network.to_string()))
}

pub fn load_config() -> Result<RpcConfig, RpcError> {
    let config: RpcConfig = toml::from_str(RPC_TOML)?;
    Ok(config)
}

pub async fn get_rpc<'a>(
    config: &'a RpcConfig,
    network: &str,
    chain: &str,
) -> Result<&'a str, RpcError> {
    let table = _validate_network(config, network)?;
    _validate_chain(table, chain, network)
}
