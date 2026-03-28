use alloy::primitives::{B256, Uint, Address};
use deterministic_deployer_evm::client::wallet_client::WalletClient;
use deterministic_deployer_evm::data::ContractSpec;
use deterministic_deployer_evm::helpers::balance_checker::check_balance;
use deterministic_deployer_evm::helpers::code_checker::has_code;
use deterministic_deployer_evm::helpers::contract_searcher::resolve_contract;
use deterministic_deployer_evm::helpers::pre_condtions::{check_before, log_info};
use deterministic_deployer_evm::types::constants::Constants;
use deterministic_deployer_evm::types::errors::{BalanceCheckerError, CliError, DeployError};
use deterministic_deployer_evm::utils::deploy::deploy_contract;
use deterministic_deployer_evm::utils::read_buf::{CliArgs, parse_args};
use std::process::exit;
use log::{error, info, warn};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    let args: CliArgs = parse_args().unwrap_or_else(|e: CliError| {
        eprintln!("Error: {e}");
        exit(1);
    });

    log_info(&args);

    let private_key: String = std::env::var(Constants::PRIVATE_KEY_ENV).unwrap_or_else(|_| {
        eprintln!(
            "Error: {} environment variable not set",
            Constants::PRIVATE_KEY_ENV
        );
        exit(1);
    });

    let contract_to_deploy: Option<&ContractSpec> = resolve_contract(&args);

    check_before(contract_to_deploy, &args);

    let mut deployers: Vec<WalletClient> = Vec::with_capacity(args.chains.len());
    for chain in &args.chains {
        match WalletClient::new(chain.network(), chain.as_rpc_key(), &private_key) {
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

    let total: usize = deployers.len();
    let mut funded: Vec<WalletClient> = Vec::with_capacity(total);

    let mut join_set: JoinSet<(WalletClient, Result<Uint<256, 4>, BalanceCheckerError>)> =
        JoinSet::new();
    for deployer in deployers {
        match has_code(&deployer, *Constants::DETERMINISTIC_DEPLOYER).await {
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

        if let Some(spec) = contract_to_deploy {
            if let Some(addr) = spec.address {
                match has_code(&deployer, addr).await {
                    Ok(true) => {
                        let chain = deployer
                            .public()
                            .map_or("unknown", |p| p.chain());
                        warn!(
                            "Contract '{}' already deployed at {addr} on {chain} — skipping",
                            spec.name
                        );
                        continue;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        warn!("Could not check contract code at {addr}: {e}");
                    }
                }
            }
        }

        join_set.spawn(async move {
            let result: Result<Uint<256, 4>, BalanceCheckerError> = check_balance(&deployer).await;
            (deployer, result)
        });
    }

    while let Some(res) = join_set.join_next().await {
        match res {
            Ok((deployer, Ok(balance))) => {
                info!("Balance for {}: {balance}", deployer.address());
                funded.push(deployer);
            }
            Ok((_deployer, Err(e))) => {
                warn!("Skipping deployer — {e}");
            }
            Err(e) => {
                warn!("Task panicked: {e}");
            }
        }
    }

    if funded.is_empty() {
        error!("No deployers with sufficient balance — aborting");
        exit(1);
    }

    if funded.len() < total {
        warn!(
            "{} of {total} deployers skipped (zero balance)",
            total - funded.len()
        );
    }

    info!("All {} deployers ready", funded.len());

    let spec: ContractSpec = match contract_to_deploy {
        Some(s) => *s,
        None => {
            warn!("No contract specified — nothing to deploy");
            return;
        }
    };

    let expected_address: Option<Address> = spec.address;

    let mut deploy_set: JoinSet<Result<(String, B256), DeployError>> = JoinSet::new();
    for deployer in funded {
        let chain = deployer
            .public()
            .map_or_else(|| "unknown".into(), |p| p.chain().to_string());
        deploy_set.spawn(async move {
            let tx_hash = deploy_contract(&deployer, &spec).await?;

            if let Some(addr) = expected_address {
                match has_code(&deployer, addr).await {
                    Ok(true) => {
                        info!("Contract verified at {addr} on {chain}");
                    }
                    Ok(false) => {
                        error!("No code at {addr} on {chain} after deploy (tx: {tx_hash})");
                    }
                    Err(e) => {
                        warn!("Could not verify code at {addr} on {chain}: {e}");
                    }
                }
            }

            Ok((chain, tx_hash))
        });
    }

    while let Some(res) = deploy_set.join_next().await {
        match res {
            Ok(Ok((chain, tx_hash))) => {
                info!("Deployed '{}' on {chain} — tx: {tx_hash}", spec.name);
            }
            Ok(Err(e)) => {
                error!("Deploy failed: {e}");
            }
            Err(e) => {
                error!("Deploy task panicked: {e}");
            }
        }
    }
}
