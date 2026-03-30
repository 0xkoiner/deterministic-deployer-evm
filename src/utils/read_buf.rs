use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::exit;

use crate::types::errors::CliError;
use alloy::primitives::{Address, B256, Bytes, U256, hex};

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
    // testnets
    Sepolia = 14,
    BaseSepolia = 15,
    ArbitrumSepolia = 16,
    OptimismSepolia = 17,
}

impl Chain {
    pub const COUNT: usize = 18;

    pub const ALL: [Self; Self::COUNT] = [
        Self::Ethereum,
        Self::Base,
        Self::Arbitrum,
        Self::Bnb,
        Self::Avalanche,
        Self::Polygon,
        Self::Sonic,
        Self::Optimism,
        Self::Zora,
        Self::ArbitrumNova,
        Self::PolygonZkevm,
        Self::Gnosis,
        Self::Scroll,
        Self::Linea,
        Self::Sepolia,
        Self::BaseSepolia,
        Self::ArbitrumSepolia,
        Self::OptimismSepolia,
    ];

    #[must_use]
    #[inline]
    pub fn from_flag(flag: &str) -> Option<Self> {
        match flag {
            "ethereum" => Some(Self::Ethereum),
            "base" => Some(Self::Base),
            "arbitrum" => Some(Self::Arbitrum),
            "bnb" => Some(Self::Bnb),
            "avalanche" => Some(Self::Avalanche),
            "polygon" => Some(Self::Polygon),
            "sonic" => Some(Self::Sonic),
            "optimism" => Some(Self::Optimism),
            "zora" => Some(Self::Zora),
            "arbitrum-nova" | "arbitrum_nova" => Some(Self::ArbitrumNova),
            "polygon-zkevm" | "polygon_zkevm" => Some(Self::PolygonZkevm),
            "gnosis" => Some(Self::Gnosis),
            "scroll" => Some(Self::Scroll),
            "linea" => Some(Self::Linea),
            "sepolia" => Some(Self::Sepolia),
            "base-sepolia" | "base_sepolia" => Some(Self::BaseSepolia),
            "arbitrum-sepolia" | "arbitrum_sepolia" => Some(Self::ArbitrumSepolia),
            "optimism-sepolia" | "optimism_sepolia" => Some(Self::OptimismSepolia),
            _ => None,
        }
    }

    #[must_use]
    #[inline]
    pub const fn as_rpc_key(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Base => "base",
            Self::Arbitrum => "arbitrum",
            Self::Bnb => "bnb",
            Self::Avalanche => "avalanche",
            Self::Polygon => "polygon",
            Self::Sonic => "sonic",
            Self::Optimism => "optimism",
            Self::Zora => "zora",
            Self::ArbitrumNova => "arbitrum_nova",
            Self::PolygonZkevm => "polygon_zkevm",
            Self::Gnosis => "gnosis",
            Self::Scroll => "scroll",
            Self::Linea => "linea",
            Self::Sepolia => "sepolia",
            Self::BaseSepolia => "base_sepolia",
            Self::ArbitrumSepolia => "arbitrum_sepolia",
            Self::OptimismSepolia => "optimism_sepolia",
        }
    }

    #[must_use]
    #[inline]
    pub const fn network(&self) -> &'static str {
        match self {
            Self::Sepolia | Self::BaseSepolia | Self::ArbitrumSepolia | Self::OptimismSepolia => {
                "testnet"
            }
            _ => "mainnet",
        }
    }

    #[must_use]
    #[inline]
    pub const fn flag(&self) -> &'static str {
        match self {
            Self::ArbitrumNova => "arbitrum-nova",
            Self::PolygonZkevm => "polygon-zkevm",
            Self::BaseSepolia => "base-sepolia",
            Self::ArbitrumSepolia => "arbitrum-sepolia",
            Self::OptimismSepolia => "optimism-sepolia",
            other => other.as_rpc_key(),
        }
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.flag())
    }
}

struct ChainSet(u32);

impl ChainSet {
    const fn new() -> Self {
        Self(0)
    }

    #[inline]
    const fn insert(&mut self, chain: Chain) -> bool {
        let bit = 1u32 << (chain as u8);
        let is_new = self.0 & bit == 0;
        self.0 |= bit;
        is_new
    }
}

fn parse_salt(input: &str) -> Result<B256, CliError> {
    if let Ok(val) = input.parse::<B256>() {
        return Ok(val);
    }

    let num: U256 = U256::from_str_radix(input, 10)
        .map_err(|e| CliError::InvalidSalt(format!("not valid hex or uint256: {e}")))?;
    Ok(B256::from(num.to_be_bytes::<32>()))
}

pub struct CliArgs {
    pub contract_path: Option<PathBuf>,
    pub chains: Vec<Chain>,
    pub salt: Option<B256>,
    pub contract_name: Option<String>,
    pub address: Option<Address>,
    pub verify: bool,
    pub constructor_args: Option<Bytes>,
}

pub fn parse_args() -> Result<CliArgs, CliError> {
    use lexopt::prelude::*;

    let mut contract_path: Option<PathBuf> = None;
    let mut salt: Option<B256> = None;
    let mut contract_name: Option<String> = None;
    let mut address: Option<Address> = None;
    let mut verify: bool = false;
    let mut constructor_args: Option<Bytes> = None;
    let mut chains: Vec<Chain> = Vec::with_capacity(Chain::COUNT);
    let mut seen: ChainSet = ChainSet::new();
    let mut parser: lexopt::Parser = lexopt::Parser::from_env();

    while let Some(arg) = parser
        .next()
        .map_err(|e| CliError::ParseError(e.to_string()))?
    {
        match arg {
            Short('h') | Long("help") => {
                print_usage();
                exit(0);
            }
            Long("salt") => {
                let val: OsString = parser
                    .value()
                    .map_err(|e| CliError::ParseError(e.to_string()))?;
                let val_str = val
                    .to_str()
                    .ok_or_else(|| CliError::InvalidSalt("invalid UTF-8".to_string()))?;
                salt = Some(parse_salt(val_str)?);
            }
            Long("contract-name") => {
                let val: OsString = parser
                    .value()
                    .map_err(|e| CliError::ParseError(e.to_string()))?;
                let val_str = val
                    .to_str()
                    .ok_or_else(|| CliError::InvalidContractName("invalid UTF-8".to_string()))?;
                contract_name = Some(val_str.to_string());
            }
            Long("verify") => {
                verify = true;
            }
            Long("constructor-args") => {
                let val: OsString = parser
                    .value()
                    .map_err(|e| CliError::ParseError(e.to_string()))?;
                let val_str = val
                    .to_str()
                    .ok_or_else(|| CliError::InvalidConstructorArgs("invalid UTF-8".to_string()))?;
                let trimmed = val_str.strip_prefix("0x").unwrap_or(val_str);
                constructor_args = Some(
                    hex::decode(trimmed)
                        .map_err(|e| CliError::InvalidConstructorArgs(e.to_string()))?
                        .into(),
                );
            }
            Long("address") => {
                let val: OsString = parser
                    .value()
                    .map_err(|e| CliError::ParseError(e.to_string()))?;
                let val_str = val
                    .to_str()
                    .ok_or_else(|| CliError::InvalidAddress("invalid UTF-8".to_string()))?;
                address = Some(
                    val_str
                        .parse::<Address>()
                        .map_err(|e| CliError::InvalidAddress(e.to_string()))?,
                );
            }
            Long(flag) => {
                let chain: Chain = Chain::from_flag(flag)
                    .ok_or_else(|| CliError::UnknownFlag(flag.to_string()))?;
                if seen.insert(chain) {
                    chains.push(chain);
                }
            }
            Value(val) if contract_path.is_none() => {
                contract_path = Some(PathBuf::from(val));
            }
            Value(_) => {
                return Err(CliError::ParseError(
                    "unexpected extra positional argument".to_string(),
                ));
            }
            Short(c) => {
                return Err(CliError::UnknownFlag(format!("-{c}")));
            }
        }
    }

    if chains.is_empty() {
        return Err(CliError::NoChainsSelected);
    }

    Ok(CliArgs {
        contract_path,
        chains,
        salt,
        contract_name,
        address,
        verify,
        constructor_args,
    })
}

fn print_usage() {
    eprintln!("Usage: dd-evm <contract.sol> --salt <hex|uint256> --chain1 [--chain2 ...]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --salt <value>  CREATE2 salt (hex bytes32 or decimal uint256)");
    eprintln!("  --contract-name <name>  Contract name (e.g. ERC20)");
    eprintln!("  --address <hex>         Contract address (hex, with or without 0x)");
    eprintln!("  --verify                Enable contract verification");
    eprintln!(
        "  --constructor-args <hex> ABI-encoded constructor arguments (hex, with or without 0x)"
    );
    eprintln!();
    eprintln!("Mainnets:");
    for chain in &Chain::ALL {
        if chain.network() == "mainnet" {
            eprintln!("  --{}", chain.flag());
        }
    }
    eprintln!();
    eprintln!("Testnets:");
    for chain in &Chain::ALL {
        if chain.network() == "testnet" {
            eprintln!("  --{}", chain.flag());
        }
    }
}
