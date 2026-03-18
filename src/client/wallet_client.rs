use crate::types::constants::Constants;
use crate::types::errors::WalletError;
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;

#[derive(Debug)]
pub struct WalletClient {
    signer: PrivateKeySigner,
}

impl WalletClient {
    pub async fn from_env() -> Result<Self, WalletError> {
        let key = std::env::var(Constants::PRIVATE_KEY_ENV)
            .map_err(|_| WalletError::EnvVarMissing(Constants::PRIVATE_KEY_ENV.to_string()))?;
        Self::from_private_key(&key).await
    }

    pub async fn from_private_key(hex: &str) -> Result<Self, WalletError> {
        let signer: PrivateKeySigner =
            hex.parse()
                .map_err(|e: alloy::signers::local::LocalSignerError| {
                    WalletError::InvalidPrivateKey(e.to_string())
                })?;
        Ok(Self { signer })
    }

    pub fn address(&self) -> Address {
        self.signer.address()
    }

    pub fn signer(&self) -> &PrivateKeySigner {
        &self.signer
    }
}
