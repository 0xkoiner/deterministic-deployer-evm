use alloy::primitives::{Address, U256};

use crate::client::public_client::PublicClient;

use crate::client::wallet_client::WalletClient;
use crate::types::errors::BalanceCheckerError;

pub async fn check_balance(wallet: &WalletClient) -> Result<U256, BalanceCheckerError> {
    let address: Address = wallet.address();

    let public: &PublicClient = wallet
        .public()
        .ok_or(BalanceCheckerError::NoProvider(address))?;

    let balance: U256 = public
        .get_balance(address)
        .await
        .map_err(|e| BalanceCheckerError::CantGetBalance(e.to_string(), address))?;

    if balance == U256::ZERO {
        return Err(BalanceCheckerError::BalanceZero(address));
    }

    Ok(balance)
}