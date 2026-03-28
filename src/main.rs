use alloy::hex;
use alloy::primitives::{Address, B256, Uint, b256};
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
use deterministic_deployer_evm::utils::verifier::verify_contract;
use log::{error, info, warn};
use std::process::exit;
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
    // for deployer in deployers {
    //     match has_code(&deployer, *Constants::DETERMINISTIC_DEPLOYER).await {
    //         Ok(true) => {
    //             info!(
    //                 "Deterministic deployer contract found for {}",
    //                 Constants::DETERMINISTIC_DEPLOYER
    //             );
    //         }
    //         Ok(false) => {
    //             warn!(
    //                 "Deterministic deployer contract NOT found for {} — skipping",
    //                 Constants::DETERMINISTIC_DEPLOYER
    //             );
    //             continue;
    //         }
    //         Err(e) => {
    //             error!(
    //                 "Failed to check deployer contract code for {}: {e}",
    //                 Constants::DETERMINISTIC_DEPLOYER
    //             );
    //             continue;
    //         }
    //     }

    //     if let Some(spec) = contract_to_deploy {
    //         if let Some(addr) = spec.address {
    //             match has_code(&deployer, addr).await {
    //                 Ok(true) => {
    //                     let chain = deployer.public().map_or("unknown", |p| p.chain());
    //                     warn!(
    //                         "Contract '{}' already deployed at {addr} on {chain} — skipping",
    //                         spec.name
    //                     );
    //                     continue;
    //                 }
    //                 Ok(false) => {}
    //                 Err(e) => {
    //                     warn!("Could not check contract code at {addr}: {e}");
    //                 }
    //             }
    //         }
    //     }

    //     join_set.spawn(async move {
    //         let result: Result<Uint<256, 4>, BalanceCheckerError> = check_balance(&deployer).await;
    //         (deployer, result)
    //     });
    // }

    // while let Some(res) = join_set.join_next().await {
    //     match res {
    //         Ok((deployer, Ok(balance))) => {
    //             info!("Balance for {}: {balance}", deployer.address());
    //             funded.push(deployer);
    //         }
    //         Ok((_deployer, Err(e))) => {
    //             warn!("Skipping deployer — {e}");
    //         }
    //         Err(e) => {
    //             warn!("Task panicked: {e}");
    //         }
    //     }
    // }

    // if funded.is_empty() {
    //     error!("No deployers with sufficient balance — aborting");
    //     exit(1);
    // }

    // if funded.len() < total {
    //     warn!(
    //         "{} of {total} deployers skipped (zero balance)",
    //         total - funded.len()
    //     );
    // }

    // info!("All {} deployers ready", funded.len());

    let spec: ContractSpec = match contract_to_deploy {
        Some(s) => *s,
        None => {
            warn!("No contract specified — nothing to deploy");
            return;
        }
    };

    let expected_address: Option<Address> = spec.address;
    let should_verify: bool = args.verify;

    let mut deploy_set: JoinSet<Result<(String, B256, WalletClient), DeployError>> = JoinSet::new();
    for deployer in deployers {
        let chain = deployer
            .public()
            .map_or_else(|| "unknown".into(), |p| p.chain().to_string());
        deploy_set.spawn(async move {
            // let tx_hash = deploy_contract(&deployer, &spec).await?;

            let tx_hash: B256 = b256!("cc73b63935b1e31a186cd339728d66edf914af3658e84e94b073d68963e52078");

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

    let mut deployed: Vec<(String, B256, WalletClient)> = Vec::new();
    while let Some(res) = deploy_set.join_next().await {
        match res {
            Ok(Ok((chain, tx_hash, deployer))) => {
                info!("Deployed '{}' on {chain} — tx: {tx_hash}", spec.name);
                deployed.push((chain, tx_hash, deployer));
            }
            Ok(Err(e)) => {
                error!("Deploy failed: {e}");
            }
            Err(e) => {
                error!("Deploy task panicked: {e}");
            }
        }
    }

    if should_verify {
        for (i, (chain, _tx_hash, deployer)) in deployed.iter().enumerate() {
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
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
}
