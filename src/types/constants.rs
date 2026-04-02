use alloy::primitives::{Address, address};
pub struct Constants;

impl Constants {
    pub const PRIVATE_KEY_ENV: &'static str = "PRIVATE_KEY";
    pub const KEYSTORE_DIR: &'static str = "src/data/keystore";
    pub const RPC_TOML: &'static str = include_str!("../data/rpc_config.toml");
    pub const DETERMINISTIC_DEPLOYER: &'static Address =
        &address!("4e59b44847b379578588920cA78FbF26c0B4956C");
    pub const EXPLORERS_TOML: &'static str = include_str!("../data/explorers_config.toml");
    pub const ETHERSCAN_API_KEY_ENV: &'static str = "ETHERSCAN_API_KEY";
    pub const SOLC_VERSION_ENV: &'static str = "SOLC_VERSION";
    pub const ETHERSCAN_V2_URL: &'static str = "https://api.etherscan.io/v2/api";
}
