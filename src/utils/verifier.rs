use std::borrow::Cow;
use std::env::var;
use std::fs::read_to_string;
use std::process::Command;
use std::process::Output;
use std::sync::Arc;
use std::time::Duration;
use serde_json::{Value, Map, from_str};

use alloy::primitives::{Address, hex};
use alloy::transports::http::reqwest;
use alloy::transports::http::reqwest::Client;
use log::{error, info, warn};
use tokio::task::{JoinSet, spawn_blocking};
use tokio::time::sleep;

use crate::types::config::{Chain, ContractSpec, EtherscanResponse, PublicClient, WalletClient, SourceCodeResult, GetSourceCodeResponse};
use crate::types::constants::Constants;
use crate::types::errors::VerifierError;

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const MAX_POLL_ATTEMPTS: u32 = 20;

fn find_contract_file(source_json: &str, contract_name: &str) -> Option<String> {
    let parsed: Value = from_str(source_json).ok()?;
    let sources: &Map<String, Value> = parsed.get("sources")?.as_object()?;
    let suffix: String = format!("{contract_name}.sol");
    sources
        .keys()
        .find(|k| k.ends_with(&suffix))
        .cloned()
}

async fn fetch_verified_source(
    api_key: &str,
    chain_id: u64,
    address: Address,
    name: &'static str,
) -> Result<SourceCodeResult, VerifierError> {
    let url: String = format!(
        "{}?module=contract&action=getsourcecode&address={address}&chainid={chain_id}&apikey={api_key}",
        Constants::ETHERSCAN_V2_URL
    );

    let client: Client = reqwest::Client::new();
    let resp: GetSourceCodeResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| VerifierError::HttpError(name, e.to_string()))?
        .json()
        .await
        .map_err(|e| VerifierError::HttpError(name, e.to_string()))?;

    if resp.status != "1" || resp.result.is_empty() {
        return Err(VerifierError::NotVerifiedOnSource(format!("chain_id {chain_id}")));
    }

    let mut source: SourceCodeResult = resp.result.into_iter().next()
        .ok_or_else(|| VerifierError::NotVerifiedOnSource(format!("chain_id {chain_id}")))?;

    if source.source_code.is_empty() || source.contract_name.is_empty() {
        return Err(VerifierError::NotVerifiedOnSource(format!("chain_id {chain_id}")));
    }

    if source.source_code.starts_with("{{") && source.source_code.ends_with("}}") {
        source.source_code = source.source_code[1..source.source_code.len() - 1].to_string();
    }

    if !source.contract_name.contains(':') {
        let contract_file: Option<String> = find_contract_file(&source.source_code, &source.contract_name);
        if let Some(file) = contract_file {
            source.contract_name = format!("{file}:{}", source.contract_name);
        }
    }

    Ok(source)
}

async fn verify_cross_chain(
    api_key: &str,
    source: &SourceCodeResult,
    target_chain_id: u64,
    address: Address,
    name: &'static str,
    chain: &str,
) -> Result<String, VerifierError> {
    let client: Client = reqwest::Client::new();
    let submit_url: String = format!(
        "{}?module=contract&action=verifysourcecode&chainid={target_chain_id}&apikey={api_key}",
        Constants::ETHERSCAN_V2_URL
    );

    let addr_str: String = format!("{address}");
    let body: String = build_form_body(&[
        ("contractaddress", &addr_str),
        ("sourceCode", &source.source_code),
        ("codeformat", "solidity-standard-json-input"),
        ("contractname", &source.contract_name),
        ("compilerversion", &source.compiler_version),
        ("constructorArguements", &source.constructor_arguments),
    ]);

    let resp: EtherscanResponse = client
        .post(&submit_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| VerifierError::HttpError(name, e.to_string()))?
        .json()
        .await
        .map_err(|e| VerifierError::HttpError(name, e.to_string()))?;

    if resp.status != "1" {
        return Err(VerifierError::SubmissionFailed(name, resp.result));
    }

    let guid: String = resp.result;
    warn!("Cross-chain verification submitted for '{name}' on {chain} (guid: {guid})");

    poll_verification_status(&client, target_chain_id, api_key, &guid, name).await
}

async fn poll_verification_status(
    client: &Client,
    chain_id: u64,
    api_key: &str,
    guid: &str,
    name: &'static str,
) -> Result<String, VerifierError> {
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
            .map_err(|e| VerifierError::HttpError(name, e.to_string()))?
            .json()
            .await
            .map_err(|e| VerifierError::HttpError(name, e.to_string()))?;

        if status.result.contains("Pass") || status.result.contains("Already Verified") {
            return Ok(status.result);
        }

        if status.result.contains("Fail") {
            return Err(VerifierError::VerificationFailed(name, status.result));
        }
    }

    Err(VerifierError::Timeout(name, guid.to_string()))
}

fn url_encode(s: &str) -> String {
    let mut out: String = String::with_capacity(s.len() * 3);
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
        ("mainnet", "plasma") => Some(9745),
        ("mainnet", "mantle") => Some(5000),
        ("mainnet", "monad") => Some(143),
        ("mainnet", "unichain") => Some(130),
        ("mainnet", "celo") => Some(42220),
        ("mainnet", "zksync") => Some(324),
        ("mainnet", "soneium") => Some(1868),
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
    contract_id: &str,
    chain_id: u64,
    api_key: &str,
    constructor_args: Option<&[u8]>,
) -> Result<String, VerifierError> {
    let addr_str: String = format!("{address}");
    let chain_str: String = chain_id.to_string();

    let mut cmd: Command = Command::new("forge");
    cmd.args([
        "verify-contract",
        &addr_str,
        contract_id,
        "--chain-id",
        &chain_str,
        "--etherscan-api-key",
        api_key,
        "--watch",
    ]);

    if let Some(cargs) = constructor_args {
        let encoded: String = hex::encode(cargs);
        cmd.args(["--constructor-args", &encoded]);
    }

    let output: Output = cmd
        .output()
        .map_err(|e| VerifierError::ForgeNotFound(e.to_string()))?;

    let stdout: Cow<'_, str> = String::from_utf8_lossy(&output.stdout);
    let stderr: Cow<'_, str> = String::from_utf8_lossy(&output.stderr);

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

    let api_key: String = var(Constants::ETHERSCAN_API_KEY_ENV)
        .map_err(|_| VerifierError::MissingEnvVar(Constants::ETHERSCAN_API_KEY_ENV))?;

    if let Some(verify_path) = spec.verify_json_path {
        verify_via_etherscan_api(
            spec,
            address,
            contract_path,
            chain,
            chain_id,
            &api_key,
            verify_path,
        )
        .await
    } else {
        let contract_id: String = format!("{contract_path}:{}", spec.name);
        let name: &str = spec.name;
        let cargs: Option<&[u8]> = spec.constructor_args;

        spawn_blocking(move || {
            verify_via_forge_sync(name, address, &contract_id, chain_id, &api_key, cargs)
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
    let compiler_version: String = var(Constants::SOLC_VERSION_ENV)
        .map_err(|_| VerifierError::MissingEnvVar(Constants::SOLC_VERSION_ENV))?;

    let source_code: String = read_to_string(verify_path)
        .map_err(|e| VerifierError::ReadFailed(spec.name, e.to_string()))?;

    let contract_name: String = format!("{contract_path}:{}", spec.name);
    let constructor_args: String = spec.constructor_args.map(hex::encode).unwrap_or_default();

    let client: Client = reqwest::Client::new();
    let submit_url: String = format!(
        "{}?module=contract&action=verifysourcecode&chainid={chain_id}&apikey={api_key}",
        Constants::ETHERSCAN_V2_URL
    );

    let addr_str: String = format!("{address}");
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

    poll_verification_status(&client, chain_id, api_key, &guid, spec.name).await
}

pub async fn run_verifications(
    ready_for_verify: &[WalletClient],
    spec: &ContractSpec,
    source_chain: Option<&str>,
) {
    if ready_for_verify.is_empty() {
        return;
    }

    info!(
        "Verifying '{}' on {} chain(s)",
        spec.name,
        ready_for_verify.len()
    );

    let name: &'static str = spec.name;
    let address: Option<Address> = spec.address;
    let path: Option<&str> = spec.path;
    let constructor_args: Option<&[u8]> = spec.constructor_args;

    let api_key: Option<String> = var(Constants::ETHERSCAN_API_KEY_ENV).ok();

    let cross_chain_source: Option<Arc<SourceCodeResult>> =
        if let (None, Some(addr), Some(key)) = (path, address, &api_key) {
            let source_chain_id: Option<u64> = source_chain.and_then(|sc| {
                let chain = Chain::from_flag(sc)?;
                etherscan_chain_id(chain.as_rpc_key(), chain.network())
            });

            if let Some(scid) = source_chain_id {
                info!("Fetching verified source from source chain (id: {scid})");
                match fetch_verified_source(key, scid, addr, name).await {
                    Ok(source) => {
                        info!(
                            "Fetched source: {} ({})",
                            source.contract_name, source.compiler_version
                        );
                        Some(Arc::new(source))
                    }
                    Err(e) => {
                        warn!("Failed to fetch source from source chain: {e}");
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

    let mut verify_set: JoinSet<()> = JoinSet::new();

    for (i, deployer) in ready_for_verify.iter().enumerate() {
        let chain: String = deployer
            .public()
            .map_or_else(|| "unknown".into(), |p| p.chain().to_string());
        let network: &str = deployer.public().map_or("mainnet", |p| p.network());
        let chain_key: &str = deployer.public().map_or("unknown", |p| p.chain());

        let chain_id: Option<u64> = etherscan_chain_id(chain_key, network);
        let api_key: Option<String> = api_key.clone();
        let delay: u64 = i as u64;
        let cross_source: Option<Arc<SourceCodeResult>> = cross_chain_source.clone();

        verify_set.spawn(async move {
            if delay > 0 {
                sleep(Duration::from_secs(delay)).await;
            }

            let result: Result<String, VerifierError> = match (address, path, chain_id, api_key) {
                (Some(addr), Some(contract_path), Some(cid), Some(key)) => {
                    let contract_id = format!("{contract_path}:{name}");
                    spawn_blocking(move || {
                        verify_via_forge_sync(name, addr, &contract_id, cid, &key, constructor_args)
                    })
                    .await
                    .map_err(|e| VerifierError::VerificationFailed(name, e.to_string()))
                    .and_then(|r| r)
                }
                (Some(addr), None, Some(cid), Some(key)) if cross_source.is_some() => {
                    let source = cross_source.unwrap();
                    verify_cross_chain(&key, &source, cid, addr, name, &chain).await
                }
                (None, _, _, _) => Err(VerifierError::MissingAddress(name)),
                (_, None, _, _) => Err(VerifierError::MissingContractPath(name)),
                (_, _, None, _) => Err(VerifierError::UnsupportedChain(chain.clone())),
                (_, _, _, None) => Err(VerifierError::MissingEnvVar(
                    Constants::ETHERSCAN_API_KEY_ENV,
                )),
            };

            match result {
                Ok(status) => info!("Verified '{name}' on {chain}: {status}"),
                Err(e) => warn!("Etherscan verification failed on {chain}: {e}"),
            }
        });
    }

    while let Some(res) = verify_set.join_next().await {
        if let Err(e) = res {
            error!("Verification task panicked: {e}");
        }
    }
}