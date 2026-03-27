use alloy::primitives::{Address, B256, keccak256};

use crate::data::contracts::ContractSpec;
use crate::types::constants::Constants;
use crate::types::errors::Create2Error;

/// Pure CREATE2 address computation — no RPC, no side effects.
#[must_use]
pub fn compute_create2_address(deployer: &Address, salt: &B256, init_code: &[u8]) -> Address {
    let init_code_hash = keccak256(init_code);

    let mut buf = [0u8; 1 + 20 + 32 + 32]; // 85 bytes
    buf[0] = 0xff;
    buf[1..21].copy_from_slice(deployer.as_slice());
    buf[21..53].copy_from_slice(salt.as_slice());
    buf[53..85].copy_from_slice(init_code_hash.as_slice());

    let hash = keccak256(buf);
    Address::from_slice(&hash[12..])
}

/// Verifies that spec.address matches the CREATE2-derived address.
/// Returns the computed address on success.
///
/// # Errors
///
/// Returns `Create2Error` if salt, init code, or address is missing,
/// or if the computed address doesn't match the expected one.
pub fn verify_create2_address(spec: &ContractSpec) -> Result<Address, Create2Error> {
    let salt = spec.salt.ok_or(Create2Error::MissingSalt(spec.name))?;

    let init_code = spec
        .full_init_code()
        .ok_or(Create2Error::MissingInitCode(spec.name))?;

    let expected = spec
        .address
        .ok_or(Create2Error::MissingAddress(spec.name))?;

    let computed =
        compute_create2_address(Constants::DETERMINISTIC_DEPLOYER, &salt, &init_code);

    if computed != expected {
        return Err(Create2Error::AddressMismatch {
            contract: spec.name,
            expected,
            computed,
        });
    }

    Ok(computed)
}
