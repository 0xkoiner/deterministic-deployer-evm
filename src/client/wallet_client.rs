use std::env::var;
use std::ops::Deref;

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;

use crate::client::public_client::PublicClient;
use crate::types::constants::Constants;
use crate::types::errors::PublicClientError;
use crate::types::errors::WalletError;

#[derive(Debug)]
pub struct WalletClient {
    signer: PrivateKeySigner,
    public: Option<PublicClient>,
}

impl WalletClient {
    pub fn from_env() -> Result<Self, WalletError> {
        let key: String = std::env::var(Constants::PRIVATE_KEY_ENV)
            .map_err(|_| WalletError::EnvVarMissing(Constants::PRIVATE_KEY_ENV))?;
        Self::from_private_key(&key)
    }

    pub fn from_private_key(hex: &str) -> Result<Self, WalletError> {
        let signer: PrivateKeySigner =
            hex.parse()
                .map_err(|e: alloy::signers::local::LocalSignerError| {
                    WalletError::InvalidPrivateKey(e.to_string())
                })?;
        Ok(Self {
            signer,
            public: None,
        })
    }

    pub async fn new(network: &str, chain: &str, private_key: &str) -> Result<Self, WalletError> {
        let signer: PrivateKeySigner =
            private_key
                .parse()
                .map_err(|e: alloy::signers::local::LocalSignerError| {
                    WalletError::InvalidPrivateKey(e.to_string())
                })?;
        let public: PublicClient = PublicClient::new_public_provider(network, chain)
            .await
            .map_err(|e: PublicClientError| WalletError::SignerError(e.to_string()))?;
        Ok(Self {
            signer,
            public: Some(public),
        })
    }

    pub async fn from_env_with_provider(network: &str, chain: &str) -> Result<Self, WalletError> {
        let key: String = var(Constants::PRIVATE_KEY_ENV)
            .map_err(|_| WalletError::EnvVarMissing(Constants::PRIVATE_KEY_ENV))?;
        Self::new(network, chain, &key).await
    }

    #[inline]
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    #[inline]
    pub fn signer(&self) -> &PrivateKeySigner {
        &self.signer
    }

    #[inline]
    pub fn public(&self) -> Option<&PublicClient> {
        self.public.as_ref()
    }
}

impl Deref for WalletClient {
    type Target = PublicClient;

    fn deref(&self) -> &PublicClient {
        self.public
            .as_ref()
            .expect("WalletClient has no provider — use new() or from_env_with_provider() instead")
    }
}
