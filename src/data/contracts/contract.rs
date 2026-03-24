use alloy::primitives::{Address, B256, Bytes};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct ContractSpec {
    pub name: &'static str,
    pub address: Option<Address>,
    pub salt: Option<B256>,
    pub path: Option<&'static str>,
    pub deployed_bytecode: Option<&'static [u8]>,
    pub constructor_args: Option<&'static [u8]>,
    pub creation_bytecode: Option<&'static [u8]>,
    pub verify_json_path: Option<&'static str>,
}

impl ContractSpec {
    /// Solidity source path as `&Path`.
    pub fn source_path(&self) -> Option<&Path> {
        self.path.map(Path::new)
    }

    /// Verify JSON path as `&Path`.
    pub fn verify_path(&self) -> Option<&Path> {
        self.verify_json_path.map(Path::new)
    }

    /// Deployed bytecode as `Bytes` (if present).
    pub fn deployed_bytes(&self) -> Option<Bytes> {
        self.deployed_bytecode.map(Bytes::copy_from_slice)
    }

    /// Constructor args as `Bytes` (if present).
    pub fn constructor_args_bytes(&self) -> Option<Bytes> {
        self.constructor_args.map(Bytes::copy_from_slice)
    }

    /// Full init code: `creation_bytecode` if set,
    /// otherwise `deployed_bytecode ++ constructor_args`.
    /// Returns `None` if neither `creation_bytecode` nor `deployed_bytecode` is set.
    pub fn full_init_code(&self) -> Option<Bytes> {
        if let Some(creation) = self.creation_bytecode {
            return Some(Bytes::copy_from_slice(creation));
        }
        let deployed = self.deployed_bytecode?;
        let mut code = deployed.to_vec();
        if let Some(args) = self.constructor_args {
            code.extend_from_slice(args);
        }
        Some(code.into())
    }
}
