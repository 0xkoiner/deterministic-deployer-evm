use alloy::primitives::{Address, B256, Bytes, FixedBytes};
use deterministic_deployer_evm::client::public_client::PublicClient;
use deterministic_deployer_evm::client::wallet_client::WalletClient;
use deterministic_deployer_evm::data::ContractSpec;
use deterministic_deployer_evm::data::contracts::build_contract_spec_from_args;
use deterministic_deployer_evm::helpers::balance_checker::check_balance;
use deterministic_deployer_evm::helpers::code_checker::has_code;
use deterministic_deployer_evm::helpers::contract_searcher::resolve_contract;
use deterministic_deployer_evm::helpers::pre_conditions::{check_before, log_info};
use deterministic_deployer_evm::types::constants::Constants;
use deterministic_deployer_evm::types::errors::{
    ArtifactError, CliError, CodeCheckerError, DeployError,
};
use deterministic_deployer_evm::utils::artifact::read_creation_bytecode;
use deterministic_deployer_evm::utils::deploy::deploy_contract;
use deterministic_deployer_evm::utils::read_buf::{Chain, CliArgs, parse_args};
use deterministic_deployer_evm::utils::verifier::verify_contract;
use env_logger::{Builder, Env};
use log::{error, info, warn};
use std::env::VarError;
use std::env::var;
use std::path::PathBuf;
use std::process::exit;
use tokio::task::JoinSet;

struct PrecheckResult {
    needs_deploy: Vec<WalletClient>,
    ready_for_verify: Vec<WalletClient>,
}

fn parse_pk() -> Result<String, VarError> {
    Ok(var(Constants::PRIVATE_KEY_ENV).unwrap_or_else(|_| {
        error!(
            "Error: {} environment variable not set",
            Constants::PRIVATE_KEY_ENV
        );
        exit(1);
    }))
}

fn create_contract_spec_from_args(args: &CliArgs) -> Option<ContractSpec> {
    let (contract_path, salt): (&PathBuf, &FixedBytes<32>) = match (&args.contract_path, &args.salt)
    {
        (Some(path), Some(salt)) => (path, salt),
        _ => return None,
    };

    let (name, creation_bytecode): (String, Bytes) =
        read_creation_bytecode(contract_path, args.contract_name.as_deref()).unwrap_or_else(
            |e: ArtifactError| {
                error!("Failed to read artifact: {e}");
                exit(1);
            },
        );

    info!("Built spec from artifact: {name}");
    Some(build_contract_spec_from_args(
        name,
        contract_path.to_string_lossy().to_string(),
        *salt,
        creation_bytecode.to_vec(),
        args.constructor_args.clone(),
    ))
}

fn create_deployers(chains: &[Chain], private_key: &str) -> Vec<WalletClient> {
    let mut deployers: Vec<WalletClient> = Vec::with_capacity(chains.len());
    for chain in chains {
        match WalletClient::new(chain.network(), chain.as_rpc_key(), private_key) {
            Ok(wallet) => {
                info!("Created deployer for {} — {}", chain, wallet.address());
                deployers.push(wallet);
            }
            Err(e) => {
                error!("Failed to create deployer for {chain}: {e}");
                exit(1);
            }
        }
    }
    deployers
}

async fn run_prechecks(deployers: Vec<WalletClient>, spec: &ContractSpec) -> PrecheckResult {
    let contract_addr: Option<Address> = spec.address;
    let mut precheck_set: JoinSet<(WalletClient, Result<bool, CodeCheckerError>, Option<bool>)> =
        JoinSet::new();

    for deployer in deployers {
        precheck_set.spawn(async move {
            let has_deployer: Result<bool, CodeCheckerError> =
                has_code(&deployer, *Constants::DETERMINISTIC_DEPLOYER).await;
            let has_contract: Option<bool> = match contract_addr {
                Some(addr) => has_code(&deployer, addr).await.ok(),
                None => None,
            };
            (deployer, has_deployer, has_contract)
        });
    }

    let mut needs_deploy: Vec<WalletClient> = Vec::new();
    let mut ready_for_verify: Vec<WalletClient> = Vec::new();

    while let Some(res) = precheck_set.join_next().await {
        let (deployer, has_deployer, has_contract) = match res {
            Ok(v) => v,
            Err(e) => {
                error!("Pre-check task panicked: {e}");
                continue;
            }
        };

        match has_deployer {
            Ok(true) => {
                info!(
                    "Deterministic deployer contract found for {}",
                    Constants::DETERMINISTIC_DEPLOYER
                );
            }
            Ok(false) => {
                warn!(
                    "Deterministic deployer contract NOT found for {} — skipping",
                    Constants::DETERMINISTIC_DEPLOYER
                );
                continue;
            }
            Err(e) => {
                error!(
                    "Failed to check deployer contract code for {}: {e}",
                    Constants::DETERMINISTIC_DEPLOYER
                );
                continue;
            }
        }

        if has_contract == Some(true) {
            let chain: &str = deployer
                .public()
                .map_or("unknown", |p: &PublicClient| p.chain());
            info!(
                "Contract '{}' already deployed at {:?} on {chain}",
                spec.name, spec.address
            );
            ready_for_verify.push(deployer);
            continue;
        }

        needs_deploy.push(deployer);
    }

    PrecheckResult {
        needs_deploy,
        ready_for_verify,
    }
}

async fn run_deployments(needs_deploy: Vec<WalletClient>, spec: ContractSpec) -> Vec<WalletClient> {
    if needs_deploy.is_empty() {
        return Vec::new();
    }

    info!("Deploying on {} chain(s)", needs_deploy.len());

    let expected_address: Option<Address> = spec.address;
    let mut deploy_set: JoinSet<Result<(String, B256, WalletClient), DeployError>> = JoinSet::new();

    for deployer in needs_deploy {
        let chain = deployer
            .public()
            .map_or_else(|| "unknown".into(), |p| p.chain().to_string());
        deploy_set.spawn(async move {
            match check_balance(&deployer).await {
                Ok(balance) => info!("Balance on {chain}: {balance}"),
                Err(e) => {
                    warn!("Skipping {chain} — {e}");
                    return Err(DeployError::NoProvider(deployer.address()));
                }
            }

            let tx_hash: FixedBytes<32> = deploy_contract(&deployer, &spec).await?;

            if let Some(addr) = expected_address {
                match has_code(&deployer, addr).await {
                    Ok(true) => {
                        info!("Contract code confirmed at {addr} on {chain}");
                    }
                    Ok(false) => {
                        error!("No code at {addr} on {chain} after deploy (tx: {tx_hash})");
                    }
                    Err(e) => {
                        warn!("Could not verify code at {addr} on {chain}: {e}");
                    }
                }
            }

            Ok((chain, tx_hash, deployer))
        });
    }

    let mut deployed: Vec<WalletClient> = Vec::new();
    while let Some(res) = deploy_set.join_next().await {
        match res {
            Ok(Ok((chain, tx_hash, deployer))) => {
                info!("Deployed '{}' on {chain} — tx: {tx_hash}", spec.name);
                deployed.push(deployer);
            }
            Ok(Err(e)) => {
                error!("Deploy failed: {e}");
            }
            Err(e) => {
                error!("Deploy task panicked: {e}");
            }
        }
    }

    deployed
}

async fn run_verifications(ready_for_verify: &[WalletClient], spec: &ContractSpec) {
    if ready_for_verify.is_empty() {
        return;
    }

    info!(
        "Verifying '{}' on {} chain(s)",
        spec.name,
        ready_for_verify.len()
    );

    for (i, deployer) in ready_for_verify.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        let chain: String = deployer
            .public()
            .map_or_else(|| "unknown".into(), |p| p.chain().to_string());
        match verify_contract(deployer, &spec).await {
            Ok(status) => {
                info!("Verified '{}' on {chain}: {status}", spec.name);
            }
            Err(e) => {
                warn!("Etherscan verification failed on {chain}: {e}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    let args: CliArgs = parse_args().unwrap_or_else(|e: CliError| {
        error!("Error: {e}");
        exit(1);
    });

    log_info(&args);

    let private_key: String = parse_pk().expect("Failed to parse private key");

    let registry_spec: Option<&ContractSpec> = resolve_contract(&args);

    let dynamic_spec: Option<ContractSpec> = if registry_spec.is_some() {
        None
    } else {
        create_contract_spec_from_args(&args)
    };

    let contract_to_deploy: Option<&ContractSpec> = registry_spec.or(dynamic_spec.as_ref());

    check_before(contract_to_deploy, &args);

    let deployers: Vec<WalletClient> = create_deployers(&args.chains, &private_key);

    let spec: ContractSpec = match contract_to_deploy {
        Some(s) => *s,
        None => {
            warn!("No contract specified — nothing to deploy");
            return;
        }
    };

    let PrecheckResult {
        needs_deploy,
        mut ready_for_verify,
    } = run_prechecks(deployers, &spec).await;

    let deployed: Vec<WalletClient> = run_deployments(needs_deploy, spec).await;
    ready_for_verify.extend(deployed);

    if args.verify {
        run_verifications(&ready_for_verify, &spec).await;
    }
}
