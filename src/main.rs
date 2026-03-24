use deterministic_deployer_evm::client::wallet_client::WalletClient;
use deterministic_deployer_evm::helpers::balance_checker::check_balance;
use deterministic_deployer_evm::types::constants::Constants;
use deterministic_deployer_evm::types::errors::CliError;
use deterministic_deployer_evm::utils::read_buf::{CliArgs, parse_args};
use log::{error, info, warn};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let args: CliArgs = parse_args().unwrap_or_else(|e: CliError| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    info!("Contract path: {}", args.contract_path.display());
    info!("Salt: {}", args.salt);

    // Read private key once, share across all deployers
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

    // Check balances — drop deployers with zero balance, keep the rest
    let total = deployers.len();
    let mut funded: Vec<WalletClient> = Vec::with_capacity(total);
    for deployer in deployers {
        match check_balance(&deployer).await {
            Ok(balance) => {
                info!("Balance for {}: {balance}", deployer.address());
                funded.push(deployer);
            }
            Err(e) => {
                warn!("Skipping deployer — {e}");
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

    let deployers = funded;
    info!("All {} deployers ready", deployers.len());
}
