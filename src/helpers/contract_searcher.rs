use crate::data::contracts::{ContractSpec, find_by_address, find_by_name, find_by_path};
use crate::utils::read_buf::CliArgs;

#[must_use]
pub fn resolve_contract(args: &CliArgs) -> Option<&'static ContractSpec> {
    if let Some(ref name) = args.contract_name {
        return find_by_name(name);
    }

    if let Some(addr) = &args.address {
        return find_by_address(addr);
    }

    if let Some(ref path) = args.contract_path {
        return find_by_path(&path.to_string_lossy());
    }

    None
}
