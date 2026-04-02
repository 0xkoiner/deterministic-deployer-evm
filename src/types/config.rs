use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct RpcConfig {
    pub mainnet: HashMap<String, String>,
    pub testnet: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct ExplorerConfig {
    pub mainnet: HashMap<String, String>,
    pub testnet: HashMap<String, String>,
}
