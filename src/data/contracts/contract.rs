use alloy::primitives::{Address, B256, Bytes, FixedBytes};
use log::{error, info};
use std::path::PathBuf;
use std::process::exit;

use crate::data::contracts::build_contract_spec_from_args;
use crate::types::errors::ArtifactError;
use crate::utils::artifact::read_creation_bytecode;
use crate::utils::read_buf::CliArgs;

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

pub fn create_contract_spec_from_args(args: &CliArgs) -> Option<ContractSpec> {
    let (contract_path, salt): (&PathBuf, &FixedBytes<32>) = match (&args.contract_path, &args.salt)
    {
        (Some(path), Some(salt)) => (path, salt),
        _ => return None,
    };

    let (name, creation_bytecode): (String, Bytes) =
        read_creation_bytecode(contract_path, args.contract_name.as_deref()).unwrap_or_else(
            |e: ArtifactError| {
                error!("Failed to read artifact: {e}");
                exit(1);
            },
        );

    info!("Built spec from artifact: {name}");
    Some(build_contract_spec_from_args(
        name,
        contract_path.to_string_lossy().to_string(),
        *salt,
        creation_bytecode.to_vec(),
        args.constructor_args.clone(),
    ))
}
