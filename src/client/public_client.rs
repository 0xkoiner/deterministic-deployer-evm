use std::borrow::Cow;
use std::fmt::Display;

use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{Address, B256, Bytes, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::rpc::types::{Block, Filter, Log, TransactionReceipt, TransactionRequest};
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::reqwest::Url;

use crate::types::config::RpcConfig;
use crate::types::errors::PublicClientError;
use crate::utils::init_rpc::{config, get_rpc};

fn map_provider_err(e: impl Display) -> PublicClientError {
    PublicClientError::ProviderError(e.to_string())
}

fn build_provider(url: &str) -> Result<DynProvider, PublicClientError> {
    let parsed: Url = url
        .parse()
        .map_err(|e| PublicClientError::InvalidUrl(format!("{e}")))?;
    Ok(ProviderBuilder::new().connect_http(parsed).erased())
}

#[derive(Debug)]
pub struct PublicClient {
    provider: DynProvider,
    chain: &'static str,
    network: &'static str,
    rpc_url: Cow<'static, str>,
}

impl PublicClient {
    pub fn new_public_provider(
        network: &'static str,
        chain: &'static str,
    ) -> Result<Self, PublicClientError> {
        Self::new_with_config(config(), network, chain)
    }

    pub fn new_with_config(
        config: &'static RpcConfig,
        network: &'static str,
        chain: &'static str,
    ) -> Result<Self, PublicClientError> {
        let rpc_url: &str =
            get_rpc(config, network, chain).map_err(PublicClientError::RpcConfig)?;
        let provider: DynProvider = build_provider(rpc_url)?;

        Ok(Self {
            provider,
            chain,
            network,
            rpc_url: Cow::Borrowed(rpc_url),
        })
    }

    pub fn new_with_signer(
        network: &'static str,
        chain: &'static str,
        signer: PrivateKeySigner,
    ) -> Result<Self, PublicClientError> {
        let rpc_url: &str =
            get_rpc(config(), network, chain).map_err(PublicClientError::RpcConfig)?;
        let parsed: Url = rpc_url
            .parse()
            .map_err(|e| PublicClientError::InvalidUrl(format!("{e}")))?;
        let provider: DynProvider = ProviderBuilder::new()
            .wallet(signer)
            .connect_http(parsed)
            .erased();
        Ok(Self {
            provider,
            chain,
            network,
            rpc_url: Cow::Borrowed(rpc_url),
        })
    }

    pub fn new_public_provider_from_url(rpc_url: &str) -> Result<Self, PublicClientError> {
        let provider: DynProvider = build_provider(rpc_url)?;

        Ok(Self {
            provider,
            chain: "",
            network: "",
            rpc_url: Cow::Owned(rpc_url.to_string()),
        })
    }

    pub async fn get_chain_id(&self) -> Result<u64, PublicClientError> {
        self.provider.get_chain_id().await.map_err(map_provider_err)
    }

    pub async fn get_block_number(&self) -> Result<u64, PublicClientError> {
        self.provider
            .get_block_number()
            .await
            .map_err(map_provider_err)
    }

    pub async fn get_gas_price(&self) -> Result<u128, PublicClientError> {
        self.provider
            .get_gas_price()
            .await
            .map_err(map_provider_err)
    }

    pub async fn get_balance(&self, address: Address) -> Result<U256, PublicClientError> {
        self.provider
            .get_balance(address)
            .await
            .map_err(map_provider_err)
    }

    pub async fn get_transaction_count(&self, address: Address) -> Result<u64, PublicClientError> {
        self.provider
            .get_transaction_count(address)
            .await
            .map_err(map_provider_err)
    }

    pub async fn get_code(&self, address: Address) -> Result<Bytes, PublicClientError> {
        self.provider
            .get_code_at(address)
            .await
            .map_err(map_provider_err)
    }

    pub async fn get_block_by_number(
        &self,
        number: BlockNumberOrTag,
    ) -> Result<Option<Block>, PublicClientError> {
        self.provider
            .get_block_by_number(number)
            .await
            .map_err(map_provider_err)
    }

    pub async fn get_transaction_receipt(
        &self,
        hash: B256,
    ) -> Result<Option<TransactionReceipt>, PublicClientError> {
        self.provider
            .get_transaction_receipt(hash)
            .await
            .map_err(map_provider_err)
    }

    pub async fn call(&self, tx: TransactionRequest) -> Result<Bytes, PublicClientError> {
        self.provider.call(tx).await.map_err(map_provider_err)
    }

    pub async fn estimate_gas(&self, tx: TransactionRequest) -> Result<u64, PublicClientError> {
        self.provider
            .estimate_gas(tx)
            .await
            .map_err(map_provider_err)
    }

    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, PublicClientError> {
        self.provider
            .get_logs(filter)
            .await
            .map_err(map_provider_err)
    }

    pub async fn get_storage_at(
        &self,
        address: Address,
        slot: U256,
    ) -> Result<U256, PublicClientError> {
        self.provider
            .get_storage_at(address, slot)
            .await
            .map_err(map_provider_err)
    }

    #[inline]
    #[must_use]
    pub const fn chain(&self) -> &str {
        self.chain
    }

    #[inline]
    #[must_use]
    pub const fn network(&self) -> &str {
        self.network
    }

    #[inline]
    #[must_use]
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    #[inline]
    #[must_use]
    pub const fn provider(&self) -> &DynProvider {
        &self.provider
    }
}
