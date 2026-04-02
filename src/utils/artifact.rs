use alloy::primitives::{Bytes, hex};
use std::path::{Path, PathBuf};

use crate::types::config::ForgeArtifact;
use crate::types::errors::ArtifactError;

pub fn read_creation_bytecode(
    contract_path: &Path,
    contract_name: Option<&str>,
) -> Result<(String, Bytes), ArtifactError> {
    let file_name: &str = contract_path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(ArtifactError::MissingFileName)?;

    let stem: &str = contract_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or(ArtifactError::MissingFileName)?;

    let name: &str = contract_name.unwrap_or(stem);

    let artifact_path: PathBuf = Path::new("out")
        .join(file_name)
        .join(format!("{name}.json"));

    let json: String = std::fs::read_to_string(&artifact_path)
        .map_err(|_| ArtifactError::NotFound(artifact_path.to_string_lossy().to_string()))?;

    let artifact: ForgeArtifact =
        serde_json::from_str(&json).map_err(|e| ArtifactError::ParseFailed(e.to_string()))?;

    let hex_str: String = artifact.bytecode.object;
    if hex_str.is_empty() || hex_str == "0x" {
        return Err(ArtifactError::EmptyBytecode(name.to_string()));
    }

    let trimmed: &str = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
    let bytes: Vec<u8> =
        hex::decode(trimmed).map_err(|e| ArtifactError::ParseFailed(e.to_string()))?;

    Ok((name.to_string(), bytes.into()))
}
