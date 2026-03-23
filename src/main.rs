mod client;
mod types;
mod utils;

use crate::utils::read_buf::parse_args;

use deterministic_deployer_evm::client::wallet_client::WalletClient;
use types::constants::Constants;
use types::errors::CliError;
use utils::read_buf::CliArgs;
use tokio::task::JoinHandle;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let args: CliArgs = parse_args().unwrap_or_else(|e: CliError| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    info!("Contract path: {}", args.contract_path.display());

    // Read private key once, share across all spawned tasks
    let private_key: String = std::env::var(Constants::PRIVATE_KEY_ENV).unwrap_or_else(|_| {
        eprintln!("Error: {} environment variable not set", Constants::PRIVATE_KEY_ENV);
        std::process::exit(1);
    });

    // Spawn all WalletClient creations in parallel
    let mut handles: Vec<JoinHandle<Result<WalletClient, deterministic_deployer_evm::types::errors::WalletError>>> = Vec::with_capacity(args.chains.len());

    for chain in &args.chains {
        let network: &'static str = chain.network();
        let chain_key: &'static str = chain.as_rpc_key();
        let pk: String = private_key.clone();

        handles.push(tokio::spawn(async move {
            WalletClient::new(network, chain_key, &pk).await
        }));
    }

    // Await in order — tasks already run concurrently
    let mut deployers: Vec<WalletClient> = Vec::with_capacity(handles.len());

    for (handle, chain) in handles.into_iter().zip(&args.chains) {
        match handle.await {
            Ok(Ok(wallet)) => {
                info!("Created deployer for {} — {}", chain, wallet.address());
                deployers.push(wallet);
            }
            Ok(Err(e)) => {
                error!("Failed to create deployer for {chain}: {e}");
                std::process::exit(1);
            }
            Err(e) => {
                error!("Task panicked for {chain}: {e}");
                std::process::exit(1);
            }
        }
    }

    info!("All {} deployers ready", deployers.len());
}
