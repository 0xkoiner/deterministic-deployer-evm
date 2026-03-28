use alloy::primitives::{Address, B256};

#[derive(Debug, thiserror::Error)]
pub enum Create2Error {
    #[error("Missing salt for contract '{0}'")]
    MissingSalt(&'static str),
    #[error("Missing init code for contract '{0}'")]
    MissingInitCode(&'static str),
    #[error("Missing expected address for contract '{0}'")]
    MissingAddress(&'static str),
    #[error("Address mismatch for '{contract}': expected {expected}, computed {computed}")]
    AddressMismatch {
        contract: &'static str,
        expected: Address,
        computed: Address,
    },
}

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
    #[error("Invalid constructor args: {0}")]
    InvalidConstructorArgs(String),
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

#[derive(Debug, thiserror::Error)]
pub enum CodeCheckerError {
    #[error("No provider attached for {0}")]
    NoProvider(Address),
    #[error("Can't fetch code for {1}: {0}")]
    ProviderError(String, Address),
}

#[derive(Debug, thiserror::Error)]
pub enum DeployError {
    #[error("No provider attached for {0}")]
    NoProvider(Address),
    #[error("Missing salt for contract '{0}'")]
    MissingSalt(&'static str),
    #[error("Missing init code for contract '{0}'")]
    MissingInitCode(&'static str),
    #[error("Simulation reverted for '{0}': {1}")]
    SimulationFailed(&'static str, String),
    #[error("Failed to send transaction for '{0}': {1}")]
    SendFailed(&'static str, String),
    #[error("Failed to get receipt for '{0}': {1}")]
    ReceiptFailed(&'static str, String),
    #[error("Transaction reverted for '{0}' (tx: {1})")]
    TxReverted(&'static str, B256),
}

#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error("No provider attached for {0}")]
    NoProvider(Address),
    #[error("Missing verify JSON path for contract '{0}'")]
    MissingVerifyPath(&'static str),
    #[error("Missing contract address for '{0}'")]
    MissingAddress(&'static str),
    #[error("Missing contract source path for '{0}'")]
    MissingContractPath(&'static str),
    #[error("Failed to read verify JSON for '{0}': {1}")]
    ReadFailed(&'static str, String),
    #[error("Missing env var: {0}")]
    MissingEnvVar(&'static str),
    #[error("Unsupported chain for verification: {0}")]
    UnsupportedChain(String),
    #[error("Verification submission failed for '{0}': {1}")]
    SubmissionFailed(&'static str, String),
    #[error("Verification failed for '{0}': {1}")]
    VerificationFailed(&'static str, String),
    #[error("Verification timed out for '{0}' (guid: {1})")]
    Timeout(&'static str, String),
    #[error("HTTP error for '{0}': {1}")]
    HttpError(&'static str, String),
    #[error("forge not found — install Foundry: {0}")]
    ForgeNotFound(String),
}
