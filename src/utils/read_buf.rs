use std::path::PathBuf;

use crate::types::errors::CliError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    // mainnets
    Ethereum,
    Base,
    Arbitrum,
    Bnb,
    Avalanche,
    Polygon,
    Sonic,
    Optimism,
    Zora,
    ArbitrumNova,
    PolygonZkevm,
    Gnosis,
    Scroll,
    Linea,
    // testnets
    Sepolia,
    BaseSepolia,
    ArbitrumSepolia,
    OptimismSepolia,
}

impl Chain {
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

    pub fn network(&self) -> &'static str {
        match self {
            Chain::Sepolia
            | Chain::BaseSepolia
            | Chain::ArbitrumSepolia
            | Chain::OptimismSepolia => "testnet",
            _ => "mainnet",
        }
    }
}

pub struct CliArgs {
    pub contract_path: PathBuf,
    pub chains: Vec<Chain>,
}

pub fn parse_args() -> Result<CliArgs, CliError> {
    use lexopt::prelude::*;

    let mut contract_path: Option<PathBuf> = None;
    let mut chains: Vec<Chain> = Vec::new();
    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser
        .next()
        .map_err(|e| CliError::ParseError(e.to_string()))?
    {
        match arg {
            Long(flag) => {
                let chain = Chain::from_flag(flag)
                    .ok_or_else(|| CliError::UnknownFlag(flag.to_string()))?;
                if !chains.contains(&chain) {
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
            Short(_) => {
                return Err(CliError::ParseError(
                    "short flags are not supported".to_string(),
                ));
            }
        }
    }

    let contract_path = contract_path.ok_or(CliError::MissingContractPath)?;

    if chains.is_empty() {
        return Err(CliError::NoChainsSelected);
    }

    Ok(CliArgs {
        contract_path,
        chains,
    })
}
