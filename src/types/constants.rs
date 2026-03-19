pub struct Constants;

impl Constants {
    pub const PRIVATE_KEY_ENV: &str = "PRIVATE_KEY";
    pub const KEYSTORE_DIR: &str = "src/data/keystore";
    pub const RPC_TOML: &str = include_str!("../data/rpc_config.toml");
}
