use log::{error, info};
use std::env::var;
use std::process::exit;

use alloy::primitives::Address;
use alloy::signers::local::{LocalSignerError, PrivateKeySigner};

use crate::types::config::{Chain, PublicClient, WalletClient};
use crate::types::constants::Constants;
use crate::types::errors::WalletError;

fn parse_signer(hex: &str) -> Result<PrivateKeySigner, WalletError> {
    hex.parse()
        .map_err(|e: LocalSignerError| WalletError::InvalidPrivateKey(e.to_string()))
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
        let public: PublicClient = PublicClient::new_with_signer(network, chain, signer.clone())
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

#[must_use]
pub fn create_deployers(chains: &[Chain], private_key: &str) -> Vec<WalletClient> {
    let mut deployers: Vec<WalletClient> = Vec::with_capacity(chains.len());
    for chain in chains {
        match WalletClient::new(chain.network(), chain.as_rpc_key(), private_key) {
            Ok(wallet) => {
                info!("Created deployer for {} — {}", chain, wallet.address());
                deployers.push(wallet);
            }
            Err(e) => {
                error!("Failed to create deployer for {chain}: {e}");
                exit(1);
            }
        }
    }
    deployers
}
