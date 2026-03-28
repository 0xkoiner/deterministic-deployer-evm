use std::time::Duration;

use alloy::primitives::{hex, Address};
use alloy::transports::http::reqwest;
use alloy::transports::http::reqwest::Client;
use log::info;
use serde::Deserialize;
use tokio::time::sleep;

use crate::client::public_client::PublicClient;
use crate::client::wallet_client::WalletClient;
use crate::data::contracts::ContractSpec;
use crate::types::constants::Constants;
use crate::types::errors::VerifierError;

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const MAX_POLL_ATTEMPTS: u32 = 20;

#[derive(Deserialize)]
struct EtherscanResponse {
    status: String,
    result: String,
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from(b"0123456789ABCDEF"[(b >> 4) as usize]));
                out.push(char::from(b"0123456789ABCDEF"[(b & 0x0F) as usize]));
            }
        }
    }
    out
}

fn build_form_body(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn etherscan_chain_id(chain: &str, network: &str) -> Option<u64> {
    match (network, chain) {
        ("mainnet", "ethereum") => Some(1),
        ("mainnet", "base") => Some(8453),
        ("mainnet", "arbitrum") => Some(42_161),
        ("mainnet", "optimism") => Some(10),
        ("mainnet", "polygon") => Some(137),
        ("mainnet", "bnb") => Some(56),
        ("mainnet", "avalanche") => Some(43_114),
        ("mainnet", "gnosis") => Some(100),
        ("mainnet", "scroll") => Some(534_352),
        ("mainnet", "linea") => Some(59_144),
        ("mainnet", "sonic") => Some(146),
        ("mainnet", "zora") => Some(7_777_777),
        ("mainnet", "arbitrum_nova") => Some(42_170),
        ("mainnet", "polygon_zkevm") => Some(1101),
        ("testnet", "sepolia") => Some(11_155_111),
        ("testnet", "base_sepolia") => Some(84_532),
        ("testnet", "arbitrum_sepolia") => Some(421_614),
        ("testnet", "optimism_sepolia") => Some(11_155_420),
        _ => None,
    }
}

fn verify_via_forge_sync(
    spec_name: &'static str,
    address: Address,
    contract_id: String,
    chain_id: u64,
    api_key: String,
    constructor_args: Option<&[u8]>,
) -> Result<String, VerifierError> {
    let addr_str = format!("{address}");
    let chain_str = chain_id.to_string();

    let mut cmd = std::process::Command::new("forge");
    cmd.args([
        "verify-contract",
        &addr_str,
        &contract_id,
        "--chain-id",
        &chain_str,
        "--etherscan-api-key",
        &api_key,
        "--watch",
    ]);

    if let Some(cargs) = constructor_args {
        let encoded = hex::encode(cargs);
        cmd.args(["--constructor-args", &encoded]);
    }

    let output = cmd
        .output()
        .map_err(|e| VerifierError::ForgeNotFound(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let filtered: String = stdout
        .lines()
        .chain(stderr.lines())
        .filter(|line| {
            !line.contains("DEBUG")
                && !line.contains("solar_")
                && !line.contains("Compiler::drop")
                && !line.trim().is_empty()
        })
        .collect::<Vec<_>>()
        .join("\n");

    if output.status.success() {
        Ok(filtered)
    } else {
        Err(VerifierError::VerificationFailed(spec_name, filtered))
    }
}

pub async fn verify_contract(
    wallet: &WalletClient,
    spec: &ContractSpec,
) -> Result<String, VerifierError> {
    let public: &PublicClient = wallet
        .public()
        .ok_or_else(|| VerifierError::NoProvider(wallet.address()))?;

    let address: Address = spec
        .address
        .ok_or(VerifierError::MissingAddress(spec.name))?;

    let contract_path: &str = spec
        .path
        .ok_or(VerifierError::MissingContractPath(spec.name))?;

    let chain: &str = public.chain();
    let network: &str = public.network();

    let chain_id: u64 = etherscan_chain_id(chain, network)
        .ok_or_else(|| VerifierError::UnsupportedChain(chain.to_string()))?;

    let api_key: String = std::env::var(Constants::ETHERSCAN_API_KEY_ENV)
        .map_err(|_| VerifierError::MissingEnvVar(Constants::ETHERSCAN_API_KEY_ENV))?;

    if let Some(verify_path) = spec.verify_json_path {
        verify_via_etherscan_api(
            spec, address, contract_path, chain, chain_id, &api_key, verify_path,
        )
        .await
    } else {
        let contract_id = format!("{contract_path}:{}", spec.name);
        let name = spec.name;
        let cargs = spec.constructor_args;

        tokio::task::spawn_blocking(move || {
            verify_via_forge_sync(name, address, contract_id, chain_id, api_key, cargs)
        })
        .await
        .map_err(|e| VerifierError::VerificationFailed(spec.name, e.to_string()))?
    }
}

async fn verify_via_etherscan_api(
    spec: &ContractSpec,
    address: Address,
    contract_path: &str,
    chain: &str,
    chain_id: u64,
    api_key: &str,
    verify_path: &str,
) -> Result<String, VerifierError> {
    let compiler_version: String = std::env::var(Constants::SOLC_VERSION_ENV)
        .map_err(|_| VerifierError::MissingEnvVar(Constants::SOLC_VERSION_ENV))?;

    let source_code: String = std::fs::read_to_string(verify_path)
        .map_err(|e| VerifierError::ReadFailed(spec.name, e.to_string()))?;

    let contract_name: String = format!("{contract_path}:{}", spec.name);
    let constructor_args: String = spec.constructor_args.map(hex::encode).unwrap_or_default();

    let client: Client = reqwest::Client::new();
    let submit_url: String = format!(
        "{}?module=contract&action=verifysourcecode&chainid={chain_id}&apikey={api_key}",
        Constants::ETHERSCAN_V2_URL
    );

    let addr_str = format!("{address}");
    let body: String = build_form_body(&[
        ("contractaddress", &addr_str),
        ("sourceCode", &source_code),
        ("codeformat", "solidity-standard-json-input"),
        ("contractname", &contract_name),
        ("compilerversion", &compiler_version),
        ("constructorArguements", &constructor_args),
    ]);

    let resp: EtherscanResponse = client
        .post(&submit_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| VerifierError::HttpError(spec.name, e.to_string()))?
        .json()
        .await
        .map_err(|e| VerifierError::HttpError(spec.name, e.to_string()))?;

    if resp.status != "1" {
        return Err(VerifierError::SubmissionFailed(spec.name, resp.result));
    }

    let guid: String = resp.result;
    info!(
        "Verification submitted for '{}' on {chain} (guid: {guid})",
        spec.name
    );

    let status_url: String = format!(
        "{}?module=contract&action=checkverifystatus&guid={guid}&chainid={chain_id}&apikey={api_key}",
        Constants::ETHERSCAN_V2_URL
    );

    for _ in 0..MAX_POLL_ATTEMPTS {
        sleep(POLL_INTERVAL).await;

        let status: EtherscanResponse = client
            .get(&status_url)
            .send()
            .await
            .map_err(|e| VerifierError::HttpError(spec.name, e.to_string()))?
            .json()
            .await
            .map_err(|e| VerifierError::HttpError(spec.name, e.to_string()))?;

        if status.result.contains("Pass") || status.result.contains("Already Verified") {
            return Ok(status.result);
        }

        if status.result.contains("Fail") {
            return Err(VerifierError::VerificationFailed(spec.name, status.result));
        }
    }

    Err(VerifierError::Timeout(spec.name, guid))
}
