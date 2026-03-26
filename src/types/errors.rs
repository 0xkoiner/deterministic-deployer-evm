use alloy::primitives::Address;

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

#[derive(Debug)]
pub enum WalletError {
    EnvVarMissing(&'static str),
    InvalidPrivateKey(String),
    SignerError(String),
    TransactionFailed(String),
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::EnvVarMissing(e) => write!(f, "Environment variable missing: {e}"),
            WalletError::InvalidPrivateKey(e) => write!(f, "Invalid private key: {e}"),
            WalletError::SignerError(e) => write!(f, "Signer error: {e}"),
            WalletError::TransactionFailed(e) => write!(f, "Transaction failed: {e}"),
        }
    }
}

impl std::error::Error for WalletError {}

#[derive(Debug)]
pub enum PublicClientError {
    RpcConfig(RpcError),
    InvalidUrl(String),
    ProviderError(String),
}

impl std::fmt::Display for PublicClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicClientError::RpcConfig(e) => write!(f, "RPC config error: {e}"),
            PublicClientError::InvalidUrl(e) => write!(f, "Invalid RPC URL: {e}"),
            PublicClientError::ProviderError(e) => write!(f, "Provider error: {e}"),
        }
    }
}

impl std::error::Error for PublicClientError {}

impl From<RpcError> for PublicClientError {
    fn from(e: RpcError) -> Self {
        PublicClientError::RpcConfig(e)
    }
}

#[derive(Debug)]
pub enum CliError {
    MissingContractPath,
    NoChainsSelected,
    UnknownFlag(String),
    ParseError(String),
    InvalidSalt(String),
    InvalidContractName(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::MissingContractPath => write!(f, "Missing contract path"),
            CliError::NoChainsSelected => write!(f, "No chains selected"),
            CliError::UnknownFlag(flag) => write!(f, "Unknown flag: --{flag}"),
            CliError::ParseError(e) => write!(f, "Parse error: {e}"),
            CliError::InvalidSalt(e) => write!(f, "Invalid salt: {e}"),
            CliError::InvalidContractName(e) => write!(f, "Invalid contract name: {e}"),
        }
    }
}

impl std::error::Error for CliError {}

#[derive(Debug)]
pub enum BalanceCheckerError {
    BalanceZero(Address),
    NoProvider(Address),
    CantGetBalance(String, Address),
}

impl std::fmt::Display for BalanceCheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalanceCheckerError::BalanceZero(address) => {
                write!(f, "Balance is zero for {address}")
            }
            BalanceCheckerError::NoProvider(address) => {
                write!(f, "No provider attached for {address}")
            }
            BalanceCheckerError::CantGetBalance(e, address) => {
                write!(f, "Can't check balance for {address}: {e}")
            }
        }
    }
}

impl std::error::Error for BalanceCheckerError {}
