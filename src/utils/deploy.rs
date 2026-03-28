use alloy::network::TransactionBuilder;
use alloy::primitives::{B256, Bytes, FixedBytes};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{TransactionReceipt, TransactionRequest};
use alloy::transports::http::reqwest::Url;
use log::warn;

use crate::client::public_client::PublicClient;
use crate::client::wallet_client::WalletClient;
use crate::data::contracts::ContractSpec;
use crate::types::constants::Constants;
use crate::types::errors::DeployError;

fn build_calldata(salt: &B256, init_code: &Bytes) -> Bytes {
    let mut buf: Vec<u8> = Vec::with_capacity(32 + init_code.len());
    buf.extend_from_slice(salt.as_slice());
    buf.extend_from_slice(init_code);
    buf.into()
}

pub async fn deploy_contract(
    wallet: &WalletClient,
    spec: &ContractSpec,
) -> Result<B256, DeployError> {
    let public: &PublicClient = wallet
        .public()
        .ok_or_else(|| DeployError::NoProvider(wallet.address()))?;

    let salt: FixedBytes<32> = spec.salt.ok_or(DeployError::MissingSalt(spec.name))?;
    let init_code: Bytes = spec
        .full_init_code()
        .ok_or(DeployError::MissingInitCode(spec.name))?;

    let calldata: Bytes = build_calldata(&salt, &init_code);

    let tx: TransactionRequest = TransactionRequest::default()
        .with_to(*Constants::DETERMINISTIC_DEPLOYER)
        .with_input(calldata);

    let gas: u64 = public
        .estimate_gas(tx.clone())
        .await
        .map_err(|e| DeployError::SimulationFailed(spec.name, e.to_string()))?;

    let tx: TransactionRequest = tx.with_gas_limit(gas);

    let url: Url = public
        .rpc_url()
        .parse()
        .map_err(|e| DeployError::SendFailed(spec.name, format!("invalid URL: {e}")))?;

    let provider = ProviderBuilder::new()
        .wallet(wallet.signer().clone())
        .connect_http(url);

    let pending = provider
        .send_transaction(tx)
        .await
        .map_err(|e| DeployError::SendFailed(spec.name, e.to_string()))?;

    let tx_hash: FixedBytes<32> = *pending.tx_hash();

    let receipt: TransactionReceipt = pending
        .get_receipt()
        .await
        .map_err(|e| DeployError::ReceiptFailed(spec.name, e.to_string()))?;

    if !receipt.status() {
        return Err(DeployError::TxReverted(spec.name, tx_hash));
    }

    warn!("TX send with {tx_hash}. Status: {}", receipt.status());

    Ok(tx_hash)
}
