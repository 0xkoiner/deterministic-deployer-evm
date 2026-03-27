use deterministic_deployer_evm::client::wallet_client::WalletClient;
use deterministic_deployer_evm::data::contracts::ContractSpec;
use deterministic_deployer_evm::helpers::balance_checker::check_balance;
use deterministic_deployer_evm::helpers::code_checker::has_code;
use deterministic_deployer_evm::helpers::contract_searcher::resolve_contract;
use deterministic_deployer_evm::types::constants::Constants;
use deterministic_deployer_evm::types::errors::CliError;
use deterministic_deployer_evm::utils::create_2::verify_create2_address;
use deterministic_deployer_evm::utils::read_buf::{CliArgs, parse_args};
use log::{error, info, warn};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    let args: CliArgs = parse_args().unwrap_or_else(|e: CliError| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    if let Some(ref contract_path) = args.contract_path {
        info!("Contract path: {}", contract_path.display());
    }
    if let Some(salt) = args.salt {
        info!("Salt: {salt}");
    }
    if let Some(ref contract_name) = args.contract_name {
        info!("Contract name: {contract_name}");
    }
    if let Some(address) = args.address {
        info!("Address: {address}");
    }
    if args.verify {
        info!("Verify: true");
    }

    let private_key: String = std::env::var(Constants::PRIVATE_KEY_ENV).unwrap_or_else(|_| {
        eprintln!(
            "Error: {} environment variable not set",
            Constants::PRIVATE_KEY_ENV
        );
        std::process::exit(1);
    });

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

    let contract_to_deploy: Option<&ContractSpec> = resolve_contract(&args);

    if let Some(spec) = contract_to_deploy {
        info!("Resolved contract: {}", spec.name);
        info!("Address contract: {:?}", spec.address);
        info!("Path contract: {:?}", spec.path);
        info!("verify_json_path contract: {:?}", spec.verify_json_path);
        info!("salt contract: {:?}", spec.salt);
    } else if args.contract_name.is_some() || args.address.is_some() || args.contract_path.is_some()
    {
        error!("Contract not found in registry");
        std::process::exit(1);
    }

    if let Some(spec) = contract_to_deploy {
        match verify_create2_address(spec) {
            Ok(addr) => {
                info!("CREATE2 address verified: {addr}");
            }
            Err(e) => {
                error!("CREATE2 verification failed: {e}");
                std::process::exit(1);
            }
        }
    }

    let mut join_set = tokio::task::JoinSet::new();
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

    let deployers: Vec<WalletClient> = funded;
    info!("All {} deployers ready", deployers.len());
}
