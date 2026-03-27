use crate::types::config::RpcConfig;
use crate::types::constants::Constants;
use crate::types::errors::RpcError;
use std::collections::HashMap;
use std::sync::OnceLock;

static RPC_CONFIG: OnceLock<RpcConfig> = OnceLock::new();

/// Returns `&'static RpcConfig`, parsing the embedded TOML exactly once.
///
/// # Panics
///
/// Panics if the embedded TOML fails to parse.
pub fn config() -> &'static RpcConfig {
    RPC_CONFIG.get_or_init(|| load_config().expect("Failed to parse embedded RPC config"))
}

fn validate_network<'a>(
    config: &'a RpcConfig,
    network: &str,
) -> Result<&'a HashMap<String, String>, RpcError> {
    match network {
        "mainnet" => Ok(&config.mainnet),
        "testnet" => Ok(&config.testnet),
        _ => Err(RpcError::UnknownNetwork(network.to_string())),
    }
}

fn validate_chain<'a>(
    table: &'a HashMap<String, String>,
    chain: &str,
    network: &str,
) -> Result<&'a str, RpcError> {
    table
        .get(chain)
        .map(String::as_str)
        .ok_or_else(|| RpcError::ChainNotFound(chain.to_string(), network.to_string()))
}

/// # Errors
///
/// Returns `RpcError` if the TOML fails to parse.
pub fn load_config() -> Result<RpcConfig, RpcError> {
    let config: RpcConfig = toml::from_str(Constants::RPC_TOML)?;
    Ok(config)
}

/// # Errors
///
/// Returns `RpcError` if the network or chain is not found.
pub fn get_rpc<'a>(
    config: &'a RpcConfig,
    network: &str,
    chain: &str,
) -> Result<&'a str, RpcError> {
    let table = validate_network(config, network)?;
    validate_chain(table, chain, network)
}
