mod client;
mod types;
mod utils;

use crate::utils::read_buf::parse_args;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let args = parse_args().unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    info!("Contract path: {}", args.contract_path.display());
    for chain in &args.chains {
        info!(
            "Chain: {} (network: {}, rpc_key: {})",
            chain.as_rpc_key(),
            chain.network(),
            chain.as_rpc_key()
        );
    }
}
