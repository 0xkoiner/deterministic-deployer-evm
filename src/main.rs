use alloy::primitives::Uint;
use deterministic_deployer_evm::client::wallet_client::WalletClient;
use deterministic_deployer_evm::data::ContractSpec;
use deterministic_deployer_evm::helpers::balance_checker::check_balance;
use deterministic_deployer_evm::helpers::code_checker::has_code;
use deterministic_deployer_evm::helpers::contract_searcher::resolve_contract;
use deterministic_deployer_evm::helpers::pre_condtions::{check_before, log_info};
use deterministic_deployer_evm::types::constants::Constants;
use deterministic_deployer_evm::types::errors::{BalanceCheckerError, CliError};
use deterministic_deployer_evm::utils::read_buf::{CliArgs, parse_args};
use log::{error, info, warn};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    let args: CliArgs = parse_args().unwrap_or_else(|e: CliError| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    log_info(&args);

    let private_key: String = std::env::var(Constants::PRIVATE_KEY_ENV).unwrap_or_else(|_| {
        eprintln!(
            "Error: {} environment variable not set",
            Constants::PRIVATE_KEY_ENV
        );
        std::process::exit(1);
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
                std::process::exit(1);
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

        join_set.spawn(async move {
            let result = check_balance(&deployer).await;
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
        std::process::exit(1);
    }

    if funded.len() < total {
        warn!(
            "{} of {total} deployers skipped (zero balance)",
            total - funded.len()
        );
    }

    info!("All {} deployers ready", funded.len());
}
