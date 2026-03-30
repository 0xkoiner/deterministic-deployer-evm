use log::{error, info, warn};
use tokio::task::JoinSet;

use alloy::network::{Ethereum, TransactionBuilder};
use alloy::primitives::{Address, B256, Bytes, FixedBytes};
use alloy::providers::{PendingTransactionBuilder, Provider};
use alloy::rpc::types::{TransactionReceipt, TransactionRequest};

use crate::client::public_client::PublicClient;
use crate::client::wallet_client::WalletClient;
use crate::data::contracts::ContractSpec;
use crate::helpers::balance_checker::check_balance;
use crate::helpers::code_checker::has_code;
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

    let pending: PendingTransactionBuilder<Ethereum> = public
        .provider()
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

    Ok(tx_hash)
}

pub async fn run_deployments(
    needs_deploy: Vec<WalletClient>,
    spec: ContractSpec,
) -> Vec<WalletClient> {
    if needs_deploy.is_empty() {
        return Vec::new();
    }

    info!("Deploying on {} chain(s)", needs_deploy.len());

    let expected_address: Option<Address> = spec.address;
    let mut deploy_set: JoinSet<Result<(String, B256, WalletClient), DeployError>> = JoinSet::new();

    for deployer in needs_deploy {
        let chain = deployer
            .public()
            .map_or_else(|| "unknown".into(), |p| p.chain().to_string());
        deploy_set.spawn(async move {
            match check_balance(&deployer).await {
                Ok(balance) => info!("Balance on {chain}: {balance}"),
                Err(e) => {
                    warn!("Skipping {chain} — {e}");
                    return Err(DeployError::NoProvider(deployer.address()));
                }
            }

            let tx_hash: FixedBytes<32> = deploy_contract(&deployer, &spec).await?;

            if let Some(addr) = expected_address {
                match has_code(&deployer, addr).await {
                    Ok(true) => {
                        info!("Contract code confirmed at {addr} on {chain}");
                    }
                    Ok(false) => {
                        error!("No code at {addr} on {chain} after deploy (tx: {tx_hash})");
                    }
                    Err(e) => {
                        warn!("Could not verify code at {addr} on {chain}: {e}");
                    }
                }
            }

            Ok((chain, tx_hash, deployer))
        });
    }

    let mut successful_deployed: Vec<WalletClient> = Vec::new();
    while let Some(res) = deploy_set.join_next().await {
        match res {
            Ok(Ok((chain, tx_hash, deployer))) => {
                info!("Deployed '{}' on {chain} — tx: {tx_hash}", spec.name);
                successful_deployed.push(deployer);
            }
            Ok(Err(e)) => {
                error!("Deploy failed: {e}");
            }
            Err(e) => {
                error!("Deploy task panicked: {e}");
            }
        }
    }

    successful_deployed
}
