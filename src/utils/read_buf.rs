use std::path::PathBuf;

use alloy::primitives::{B256, U256};

use crate::types::errors::CliError;

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

    pub const ALL: [Chain; Self::COUNT] = [
        Chain::Ethereum,
        Chain::Base,
        Chain::Arbitrum,
        Chain::Bnb,
        Chain::Avalanche,
        Chain::Polygon,
        Chain::Sonic,
        Chain::Optimism,
        Chain::Zora,
        Chain::ArbitrumNova,
        Chain::PolygonZkevm,
        Chain::Gnosis,
        Chain::Scroll,
        Chain::Linea,
        Chain::Sepolia,
        Chain::BaseSepolia,
        Chain::ArbitrumSepolia,
        Chain::OptimismSepolia,
    ];

    #[must_use]
    #[inline]
    pub fn from_flag(flag: &str) -> Option<Self> {
        match flag {
            "ethereum" => Some(Chain::Ethereum),
            "base" => Some(Chain::Base),
            "arbitrum" => Some(Chain::Arbitrum),
            "bnb" => Some(Chain::Bnb),
            "avalanche" => Some(Chain::Avalanche),
            "polygon" => Some(Chain::Polygon),
            "sonic" => Some(Chain::Sonic),
            "optimism" => Some(Chain::Optimism),
            "zora" => Some(Chain::Zora),
            "arbitrum-nova" => Some(Chain::ArbitrumNova),
            "polygon-zkevm" => Some(Chain::PolygonZkevm),
            "gnosis" => Some(Chain::Gnosis),
            "scroll" => Some(Chain::Scroll),
            "linea" => Some(Chain::Linea),
            "sepolia" => Some(Chain::Sepolia),
            "base-sepolia" => Some(Chain::BaseSepolia),
            "arbitrum-sepolia" => Some(Chain::ArbitrumSepolia),
            "optimism-sepolia" => Some(Chain::OptimismSepolia),
            _ => None,
        }
    }

    #[must_use]
    #[inline]
    pub fn as_rpc_key(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Base => "base",
            Chain::Arbitrum => "arbitrum",
            Chain::Bnb => "bnb",
            Chain::Avalanche => "avalanche",
            Chain::Polygon => "polygon",
            Chain::Sonic => "sonic",
            Chain::Optimism => "optimism",
            Chain::Zora => "zora",
            Chain::ArbitrumNova => "arbitrum_nova",
            Chain::PolygonZkevm => "polygon_zkevm",
            Chain::Gnosis => "gnosis",
            Chain::Scroll => "scroll",
            Chain::Linea => "linea",
            Chain::Sepolia => "sepolia",
            Chain::BaseSepolia => "base_sepolia",
            Chain::ArbitrumSepolia => "arbitrum_sepolia",
            Chain::OptimismSepolia => "optimism_sepolia",
        }
    }

    #[must_use]
    #[inline]
    pub fn network(&self) -> &'static str {
        match self {
            Chain::Sepolia
            | Chain::BaseSepolia
            | Chain::ArbitrumSepolia
            | Chain::OptimismSepolia => "testnet",
            _ => "mainnet",
        }
    }

    #[must_use]
    #[inline]
    pub fn flag(&self) -> &'static str {
        match self {
            Chain::ArbitrumNova => "arbitrum-nova",
            Chain::PolygonZkevm => "polygon-zkevm",
            Chain::BaseSepolia => "base-sepolia",
            Chain::ArbitrumSepolia => "arbitrum-sepolia",
            Chain::OptimismSepolia => "optimism-sepolia",
            other => other.as_rpc_key(),
        }
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.flag())
    }
}

struct ChainSet(u32);

impl ChainSet {
    const fn new() -> Self {
        Self(0)
    }

    #[inline]
    fn insert(&mut self, chain: Chain) -> bool {
        let bit: u32 = 1u32 << (chain as u8);
        let is_new: bool = self.0 & bit == 0;
        self.0 |= bit;
        is_new
    }
}

// ── Salt Parsing ───────────────────────────────────────────

fn parse_salt(input: &str) -> Result<B256, CliError> {
    // Try hex first (with or without 0x prefix)
    if let Ok(val) = input.parse::<B256>() {
        return Ok(val);
    }

    // Try decimal uint256 → B256
    let num = U256::from_str_radix(input, 10)
        .map_err(|e| CliError::InvalidSalt(format!("not valid hex or uint256: {e}")))?;
    Ok(B256::from(num.to_be_bytes::<32>()))
}

// ── CLI Args ────────────────────────────────────────────

pub struct CliArgs {
    pub contract_path: PathBuf,
    pub chains: Vec<Chain>,
    pub salt: Option<B256>,
}

pub fn parse_args() -> Result<CliArgs, CliError> {
    use lexopt::prelude::*;

    let mut contract_path: Option<PathBuf> = None;
    let mut salt: Option<B256> = None;
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
                std::process::exit(0);
            }
            Long("salt") => {
                let val = parser
                    .value()
                    .map_err(|e| CliError::ParseError(e.to_string()))?;
                let val_str = val
                    .to_str()
                    .ok_or_else(|| CliError::InvalidSalt("invalid UTF-8".to_string()))?;
                salt = Some(parse_salt(val_str)?);
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

    let contract_path: PathBuf = contract_path.ok_or(CliError::MissingContractPath)?;

    if chains.is_empty() {
        return Err(CliError::NoChainsSelected);
    }

    Ok(CliArgs {
        contract_path,
        chains,
        salt,
    })
}

fn print_usage() {
    eprintln!("Usage: dd-evm <contract.sol> --salt <hex|uint256> --chain1 [--chain2 ...]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --salt <value>  CREATE2 salt (hex bytes32 or decimal uint256)");
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
