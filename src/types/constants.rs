use alloy::primitives::{Address, address};
pub struct Constants;

impl Constants {
    pub const PRIVATE_KEY_ENV: &'static str = "PRIVATE_KEY";
    pub const KEYSTORE_DIR: &'static str = "src/data/keystore";
    pub const RPC_TOML: &'static str = include_str!("../data/rpc_config.toml");
    pub const DETERMINISTIC_DEPLOYER: &'static Address =
        &address!("4e59b44847b379578588920cA78FbF26c0B4956C");
}
