#[derive(Debug)]
pub enum RpcError {
    IoError(std::io::Error),       // wraps the real I/O error
    ParseError(toml::de::Error),   // wraps the real TOML parse error
    UnknownNetwork(String),        // "mainnet" or "testnet" only
    ChainNotFound(String, String), // (chain, network)
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcError::IoError(e) => write!(f, "Failed to read config: {e}"),
            RpcError::ParseError(e) => write!(f, "Failed to parse TOML: {e}"),
            RpcError::UnknownNetwork(n) => write!(f, "Unknown network: {n}"),
            RpcError::ChainNotFound(chain, network) => {
                write!(f, "Chain {chain} not found in {network}")
            }
        }
    }
}

impl std::error::Error for RpcError {}

impl From<std::io::Error> for RpcError {
    fn from(e: std::io::Error) -> Self {
        RpcError::IoError(e)
    }
}

impl From<toml::de::Error> for RpcError {
    fn from(e: toml::de::Error) -> Self {
        RpcError::ParseError(e)
    }
}
