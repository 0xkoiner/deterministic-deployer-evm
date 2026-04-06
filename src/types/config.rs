use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

use alloy::primitives::{Address, B256, Bytes};
use alloy::providers::DynProvider;
use alloy::signers::local::PrivateKeySigner;

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

#[derive(Debug)]
pub struct PublicClient {
    pub provider: DynProvider,
    pub chain: &'static str,
    pub network: &'static str,
    pub rpc_url: Cow<'static, str>,
}

#[derive(Debug)]
pub struct WalletClient {
    pub signer: PrivateKeySigner,
    pub public: Option<PublicClient>,
}

#[derive(Debug, Clone, Copy)]
pub struct ContractSpec {
    pub name: &'static str,
    pub address: Option<Address>,
    pub salt: Option<B256>,
    pub path: Option<&'static str>,
    pub deployer_tx: Option<&'static [u8]>,
    pub constructor_args: Option<&'static [u8]>,
    pub creation_bytecode: Option<&'static [u8]>,
    pub verify_json_path: Option<&'static str>,
}

#[derive(Deserialize)]
pub struct BytecodeObject {
    pub object: String,
}

#[derive(Deserialize)]
pub struct ForgeArtifact {
    pub bytecode: BytecodeObject,
}

#[derive(Deserialize)]
pub struct EtherscanResponse {
    pub status: String,
    pub result: String,
}

#[derive(Deserialize)]
pub struct SourceCodeResult {
    #[serde(rename = "SourceCode")]
    pub source_code: String,
    #[serde(rename = "ContractName")]
    pub contract_name: String,
    #[serde(rename = "CompilerVersion")]
    pub compiler_version: String,
    #[serde(rename = "ConstructorArguments")]
    pub constructor_arguments: String,
}

#[derive(Deserialize)]
pub struct GetSourceCodeResponse {
    pub status: String,
    pub result: Vec<SourceCodeResult>,
}

pub struct CliArgs {
    pub contract_path: Option<PathBuf>,
    pub chains: Vec<Chain>,
    pub salt: Option<B256>,
    pub contract_name: Option<String>,
    pub address: Option<Address>,
    pub verify: bool,
    pub constructor_args: Option<Bytes>,
    pub keystore: bool,
    pub source_chain: Option<String>,
}

pub struct PrecheckResult {
    pub needs_deploy: Vec<WalletClient>,
    pub ready_for_verify: Vec<WalletClient>,
}

pub struct ChainSet(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Chain {
    // mainnets
    Ethereum = 0,
    Base = 1,
    Arbitrum = 2,
    Bnb = 3,
    Avalanche = 4,
    Polygon = 5,
    Sonic = 6,
    Optimism = 7,
    Zora = 8,
    ArbitrumNova = 9,
    PolygonZkevm = 10,
    Gnosis = 11,
    Scroll = 12,
    Linea = 13,
    Plasma = 14,
    Mantle = 15,
    Monad = 16,
    Unichain = 17,
    Celo = 18,
    Zksync = 19,
    Soneium = 20,
    // testnets
    Sepolia = 21,
    BaseSepolia = 22,
    ArbitrumSepolia = 23,
    OptimismSepolia = 24,
}
