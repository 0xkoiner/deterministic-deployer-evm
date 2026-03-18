pub struct Constants;

impl Constants {
    pub const RPC_TOML: &str = include_str!("../data/rpc_config.toml");
    pub const PRIVATE_KEY_ENV: &str = "PRIVATE_KEY";
}
