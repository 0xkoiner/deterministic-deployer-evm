use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct RpcConfig {
    pub mainnet     : HashMap<String, String>, // {chain_name: rpc_url}
    pub testnet     : HashMap<String, String>, // {chain_name: rpc_url}
}