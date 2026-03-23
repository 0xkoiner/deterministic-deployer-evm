mod client;
mod types;
mod utils;

use crate::utils::read_buf::parse_args;
use deterministic_deployer_evm::client::wallet_client::WalletClient;
use types::errors::CliError;
use utils::read_buf::CliArgs;

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

    let mut deployers: Vec<WalletClient>;

    // for chain in &args.chains {
        
    //     info!(
    //         "Chain: {} (network: {}, rpc_key: {})",
    //         chain.as_rpc_key(),
    //         chain.network(),
    //         chain.as_rpc_key()
    //     );
    // }
}
