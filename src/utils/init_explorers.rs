use crate::types::config::ExplorerConfig;
use crate::types::constants::Constants;
use crate::types::errors::ExplorerError;
use std::collections::HashMap;
use std::sync::OnceLock;

static EXPLORER_CONFIG: OnceLock<ExplorerConfig> = OnceLock::new();

pub fn explorer_config() -> &'static ExplorerConfig {
    EXPLORER_CONFIG.get_or_init(|| load_config().expect("Failed to parse embedded explorer config"))
}

fn validate_network<'a>(
    config: &'a ExplorerConfig,
    network: &str,
) -> Result<&'a HashMap<String, String>, ExplorerError> {
    match network {
        "mainnet" => Ok(&config.mainnet),
        "testnet" => Ok(&config.testnet),
        _ => Err(ExplorerError::UnknownNetwork(network.to_string())),
    }
}

fn validate_chain<'a>(
    table: &'a HashMap<String, String>,
    chain: &str,
    network: &str,
) -> Result<&'a str, ExplorerError> {
    table
        .get(chain)
        .map(String::as_str)
        .ok_or_else(|| ExplorerError::ChainNotFound(chain.to_string(), network.to_string()))
}

pub fn load_config() -> Result<ExplorerConfig, ExplorerError> {
    let config: ExplorerConfig = toml::from_str(Constants::EXPLORERS_TOML)?;
    Ok(config)
}

pub fn get_explorer<'a>(
    config: &'a ExplorerConfig,
    network: &str,
    chain: &str,
) -> Result<&'a str, ExplorerError> {
    let table: &HashMap<String, String> = validate_network(config, network)?;
    validate_chain(table, chain, network)
}

pub fn tx_url(network: &str, chain: &str, tx_hash: &str) -> Option<String> {
    let config: &ExplorerConfig = explorer_config();
    get_explorer(config, network, chain)
        .ok()
        .map(|base| format!("{base}tx/{tx_hash}"))
}

pub fn addr_url(network: &str, chain: &str, address: &str) -> Option<String> {
    let config: &ExplorerConfig = explorer_config();
    get_explorer(config, network, chain)
        .ok()
        .map(|base| format!("{base}address/{address}"))
    
}
