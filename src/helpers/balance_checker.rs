use alloy::primitives::U256;

use crate::client::wallet_client::WalletClient;
use crate::types::errors::BalanceCheckerError;

/// Checks that the wallet has a non-zero native balance on its connected chain.
/// Returns the balance on success, or a hard error if zero or unreachable.
pub async fn check_balance(wallet: &WalletClient) -> Result<U256, BalanceCheckerError> {
    let address = wallet.address();

    let public = wallet.public().ok_or_else(|| {
        BalanceCheckerError::CantGetBalance(
            "WalletClient has no provider — use new() or from_env_with_provider()".to_string(),
            address,
        )
    })?;

    let balance = public
        .get_balance(address)
        .await
        .map_err(|e| BalanceCheckerError::CantGetBalance(e.to_string(), address))?;

    if balance == U256::ZERO {
        return Err(BalanceCheckerError::BalanceZero(address));
    }

    Ok(balance)
}
