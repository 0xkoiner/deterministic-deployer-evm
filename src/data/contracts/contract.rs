use alloy::primitives::{Address, B256, Bytes};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct ContractSpec {
    pub name: &'static str,
    pub address: Option<Address>,
    pub salt: Option<B256>,
    pub path: Option<&'static str>,
    pub deployer_tx: Option<&'static [u8]>,
    pub constructor_args: Option<&'static [u8]>,
    pub creation_bytecode: Option<&'static [u8]>,
    pub verify_json_path: Option<&'static str>,
}

impl ContractSpec {
    pub fn source_path(&self) -> Option<&Path> {
        self.path.map(Path::new)
    }

    pub fn verify_path(&self) -> Option<&Path> {
        self.verify_json_path.map(Path::new)
    }

    pub fn deployed_bytes(&self) -> Option<Bytes> {
        self.deployer_tx.map(Bytes::copy_from_slice)
    }

    pub fn constructor_args_bytes(&self) -> Option<Bytes> {
        self.constructor_args.map(Bytes::copy_from_slice)
    }

    pub fn full_init_code(&self) -> Option<Bytes> {
        let creation: &[u8] = self.creation_bytecode?;
        let args_len: usize = self.constructor_args.map_or(0, <[u8]>::len);
        let mut code: Vec<u8> = Vec::with_capacity(creation.len() + args_len);
        code.extend_from_slice(creation);
        if let Some(args) = self.constructor_args {
            code.extend_from_slice(args);
        }
        Some(code.into())
    }
}
