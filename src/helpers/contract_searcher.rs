use crate::data::contracts::{ContractSpec, find_by_address, find_by_name, find_by_path};
use crate::utils::read_buf::CliArgs;

pub fn resolve_contract(args: &CliArgs) -> Option<&'static ContractSpec> {
    // Priority 1: --contract-name
    if let Some(ref name) = args.contract_name {
        return find_by_name(name);
    }

    // Priority 2: --address
    if let Some(ref addr) = args.address {
        return find_by_address(addr);
    }

    // Priority 3: positional path argument
    if let Some(ref path) = args.contract_path {
        return find_by_path(&path.to_string_lossy());
    }

    None
}
