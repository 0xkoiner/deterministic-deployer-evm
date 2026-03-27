use alloy::primitives::Address;

use crate::client::public_client::PublicClient;

use crate::client::wallet_client::WalletClient;
use crate::types::errors::CodeCheckerError;

pub async fn has_code(wallet: &WalletClient, address: Address) -> Result<bool, CodeCheckerError> {
    let public: &PublicClient = wallet
        .public()
        .ok_or(CodeCheckerError::NoProvider(address))?;

    let code = public
        .get_code(address)
        .await
        .map_err(|e| CodeCheckerError::ProviderError(e.to_string(), address))?;

    Ok(!code.is_empty())
}
