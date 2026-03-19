use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::rpc::types::{Block, Filter, Log, TransactionReceipt, TransactionRequest};

use crate::types::config::RpcConfig;
use crate::types::errors::PublicClientError;
use crate::utils::init_rpc::{get_rpc, load_config};

#[derive(Debug)]
pub struct PublicClient {
    provider: DynProvider,
    chain: String,
    network: String,
    rpc_url: String,
}

impl PublicClient {
    pub async fn new_public_provider(network: &str, chain: &str) -> Result<Self, PublicClientError> {
        let config:RpcConfig = load_config().map_err(PublicClientError::RpcConfig)?;
        let rpc_url: &str = get_rpc(&config, network, chain)
            .await
            .map_err(PublicClientError::RpcConfig)?;

        let provider: DynProvider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| {
                PublicClientError::InvalidUrl(format!("{e}"))
            })?)
            .erased();

        Ok(Self {
            provider,
            chain: chain.to_string(),
            network: network.to_string(),
            rpc_url: rpc_url.to_string(),
        })
    }

    pub fn new_public_provider_from_url(rpc_url: &str) -> Result<Self, PublicClientError> {
        let provider: DynProvider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| {
                PublicClientError::InvalidUrl(format!("{e}"))
            })?)
            .erased();

        Ok(Self {
            provider,
            chain: String::new(),
            network: String::new(),
            rpc_url: rpc_url.to_string(),
        })
    }

    pub async fn get_chain_id(&self) -> Result<u64, PublicClientError> {
        self.provider
            .get_chain_id()
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_block_number(&self) -> Result<u64, PublicClientError> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_gas_price(&self) -> Result<u128, PublicClientError> {
        self.provider
            .get_gas_price()
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_balance(&self, address: Address) -> Result<U256, PublicClientError> {
        self.provider
            .get_balance(address)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_transaction_count(&self, address: Address) -> Result<u64, PublicClientError> {
        self.provider
            .get_transaction_count(address)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_code(&self, address: Address) -> Result<Bytes, PublicClientError> {
        self.provider
            .get_code_at(address)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_block_by_number(
        &self,
        number: BlockNumberOrTag,
    ) -> Result<Option<Block>, PublicClientError> {
        self.provider
            .get_block_by_number(number)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_transaction_receipt(
        &self,
        hash: B256,
    ) -> Result<Option<TransactionReceipt>, PublicClientError> {
        self.provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn call(&self, tx: TransactionRequest) -> Result<Bytes, PublicClientError> {
        self.provider
            .call(tx)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn estimate_gas(&self, tx: TransactionRequest) -> Result<u64, PublicClientError> {
        self.provider
            .estimate_gas(tx)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, PublicClientError> {
        self.provider
            .get_logs(filter)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    pub async fn get_storage_at(
        &self,
        address: Address,
        slot: U256,
    ) -> Result<U256, PublicClientError> {
        self.provider
            .get_storage_at(address, slot)
            .await
            .map_err(|e| PublicClientError::ProviderError(e.to_string()))
    }

    #[inline]
    pub fn chain(&self) -> &str {
        &self.chain
    }

    #[inline]
    pub fn network(&self) -> &str {
        &self.network
    }

    #[inline]
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    #[inline]
    pub fn provider(&self) -> &DynProvider {
        &self.provider
    }
}
