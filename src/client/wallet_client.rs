use std::env::var;

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;

use crate::client::public_client::PublicClient;
use crate::types::constants::Constants;
use crate::types::errors::WalletError;

fn parse_signer(hex: &str) -> Result<PrivateKeySigner, WalletError> {
    hex.parse()
        .map_err(|e: alloy::signers::local::LocalSignerError| {
            WalletError::InvalidPrivateKey(e.to_string())
        })
}

#[derive(Debug)]
pub struct WalletClient {
    signer: PrivateKeySigner,
    public: Option<PublicClient>,
}

impl WalletClient {
    pub fn from_env() -> Result<Self, WalletError> {
        let key: String = var(Constants::PRIVATE_KEY_ENV)
            .map_err(|_| WalletError::EnvVarMissing(Constants::PRIVATE_KEY_ENV))?;
        Self::from_private_key(&key)
    }

    pub fn from_private_key(hex: &str) -> Result<Self, WalletError> {
        let signer = parse_signer(hex)?;
        Ok(Self {
            signer,
            public: None,
        })
    }

    pub fn new(
        network: &'static str,
        chain: &'static str,
        private_key: &str,
    ) -> Result<Self, WalletError> {
        let signer = parse_signer(private_key)?;
        let public = PublicClient::new_with_signer(network, chain, signer.clone())
            .map_err(|e| WalletError::SignerError(e.to_string()))?;
        Ok(Self {
            signer,
            public: Some(public),
        })
    }

    #[inline]
    #[must_use]
    pub const fn address(&self) -> Address {
        self.signer.address()
    }

    #[inline]
    #[must_use]
    pub const fn signer(&self) -> &PrivateKeySigner {
        &self.signer
    }

    #[inline]
    #[must_use]
    pub const fn public(&self) -> Option<&PublicClient> {
        self.public.as_ref()
    }
}
