use std::path::Path;

use alloy::primitives::{Bytes, hex};
use serde::Deserialize;

use crate::types::errors::ArtifactError;

#[derive(Deserialize)]
struct BytecodeObject {
    object: String,
}

#[derive(Deserialize)]
struct ForgeArtifact {
    bytecode: BytecodeObject,
}

pub fn read_creation_bytecode(
    contract_path: &Path,
    contract_name: Option<&str>,
) -> Result<(String, Bytes), ArtifactError> {
    let file_name = contract_path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(ArtifactError::MissingFileName)?;

    let stem = contract_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or(ArtifactError::MissingFileName)?;

    let name = contract_name.unwrap_or(stem);

    let artifact_path = Path::new("out")
        .join(file_name)
        .join(format!("{name}.json"));

    let json = std::fs::read_to_string(&artifact_path)
        .map_err(|_| ArtifactError::NotFound(artifact_path.to_string_lossy().to_string()))?;

    let artifact: ForgeArtifact =
        serde_json::from_str(&json).map_err(|e| ArtifactError::ParseFailed(e.to_string()))?;

    let hex_str = artifact.bytecode.object;
    if hex_str.is_empty() || hex_str == "0x" {
        return Err(ArtifactError::EmptyBytecode(name.to_string()));
    }

    let trimmed = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
    let bytes = hex::decode(trimmed).map_err(|e| ArtifactError::ParseFailed(e.to_string()))?;

    Ok((name.to_string(), bytes.into()))
}
