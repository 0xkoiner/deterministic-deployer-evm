use log::{error, info};

use crate::data::contracts::ContractSpec;
use crate::utils::create_2::verify_create2_address;
use crate::utils::read_buf::CliArgs;

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
}

pub fn check_before(contract_to_deploy: &Option<&ContractSpec>, args: &CliArgs) {
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
}
