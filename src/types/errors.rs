use alloy::primitives::Address;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("Failed to read config: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Unknown network: {0}")]
    UnknownNetwork(String),
    #[error("Chain {0} not found in {1}")]
    ChainNotFound(String, String),
}

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Environment variable missing: {0}")]
    EnvVarMissing(&'static str),
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Signer error: {0}")]
    SignerError(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PublicClientError {
    #[error("RPC config error: {0}")]
    RpcConfig(#[from] RpcError),
    #[error("Invalid RPC URL: {0}")]
    InvalidUrl(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Missing contract path")]
    MissingContractPath,
    #[error("No chains selected")]
    NoChainsSelected,
    #[error("Unknown flag: --{0}")]
    UnknownFlag(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid salt: {0}")]
    InvalidSalt(String),
    #[error("Invalid contract name: {0}")]
    InvalidContractName(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

#[derive(Debug, thiserror::Error)]
pub enum BalanceCheckerError {
    #[error("Balance is zero for {0}")]
    BalanceZero(Address),
    #[error("No provider attached for {0}")]
    NoProvider(Address),
    #[error("Can't check balance for {1}: {0}")]
    CantGetBalance(String, Address),
}
