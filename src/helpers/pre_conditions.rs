use log::{error, info, warn};
use std::process::exit;
use tokio::task::JoinSet;

use alloy::primitives::Address;

use crate::client::public_client::PublicClient;
use crate::client::wallet_client::WalletClient;
use crate::data::contracts::ContractSpec;
use crate::helpers::code_checker::has_code;
use crate::types::constants::Constants;
use crate::types::errors::CodeCheckerError;
use crate::utils::create_2::verify_create2_address;
use crate::utils::read_buf::CliArgs;

pub struct PrecheckResult {
    pub needs_deploy: Vec<WalletClient>,
    pub ready_for_verify: Vec<WalletClient>,
}

pub fn log_info(args: &CliArgs) {
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
    if let Some(ref constructor_args) = args.constructor_args {
        info!("Constructor args: {constructor_args}");
    }
}

pub fn check_before(contract_to_deploy: Option<&ContractSpec>, args: &CliArgs) {
    if let Some(spec) = contract_to_deploy {
        info!("Resolved contract: {}", spec.name);
        info!("Address contract: {:?}", spec.address);
        info!("Path contract: {:?}", spec.path);
        info!("verify_json_path contract: {:?}", spec.verify_json_path);
        info!("salt contract: {:?}", spec.salt);

        match verify_create2_address(spec) {
            Ok(addr) => {
                info!("CREATE2 address verified: {addr}");
            }
            Err(e) => {
                error!("CREATE2 verification failed: {e}");
                exit(1);
            }
        }
    } else if args.contract_name.is_some() || args.address.is_some() || args.contract_path.is_some()
    {
        error!("Contract not found in registry");
        exit(1);
    }
}

pub async fn run_prechecks(deployers: Vec<WalletClient>, spec: &ContractSpec) -> PrecheckResult {
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
