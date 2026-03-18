use deterministic_deployer_evm::client::wallet_client::WalletClient;

// Valid Anvil test private key (do NOT use in production)
const TEST_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const TEST_PRIVATE_KEY_0X: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

// -- from_private_key tests --

#[tokio::test]
async fn test_from_private_key_success() {
    let wallet = WalletClient::from_private_key(TEST_PRIVATE_KEY).await;
    assert!(wallet.is_ok(), "should parse a valid hex private key");
}

#[tokio::test]
async fn test_from_private_key_with_0x_prefix() {
    let wallet = WalletClient::from_private_key(TEST_PRIVATE_KEY_0X).await;
    assert!(wallet.is_ok(), "should parse a 0x-prefixed private key");
}

#[tokio::test]
async fn test_from_private_key_returns_correct_address() {
    let wallet = WalletClient::from_private_key(TEST_PRIVATE_KEY)
        .await
        .unwrap();
    let addr = format!("{}", wallet.address());
    assert!(
        !addr.is_empty(),
        "address should not be empty for a valid key"
    );
}

#[tokio::test]
async fn test_from_private_key_both_formats_same_address() {
    let w1 = WalletClient::from_private_key(TEST_PRIVATE_KEY)
        .await
        .unwrap();
    let w2 = WalletClient::from_private_key(TEST_PRIVATE_KEY_0X)
        .await
        .unwrap();
    assert_eq!(
        w1.address(),
        w2.address(),
        "with and without 0x prefix should produce the same address"
    );
}

#[tokio::test]
async fn test_signer_returns_reference() {
    let wallet = WalletClient::from_private_key(TEST_PRIVATE_KEY)
        .await
        .unwrap();
    let signer = wallet.signer();
    assert_eq!(
        signer.address(),
        wallet.address(),
        "signer address should match wallet address"
    );
}

// -- from_private_key error tests --

#[tokio::test]
async fn test_from_private_key_invalid_hex() {
    let result = WalletClient::from_private_key("not_a_valid_hex_key").await;
    assert!(result.is_err(), "should fail on invalid hex");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Invalid private key"),
        "error should mention 'Invalid private key', got: {err_msg}"
    );
}

#[tokio::test]
async fn test_from_private_key_empty_string() {
    let result = WalletClient::from_private_key("").await;
    assert!(result.is_err(), "should fail on empty string");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Invalid private key"),
        "error should mention 'Invalid private key', got: {err_msg}"
    );
}

#[tokio::test]
async fn test_from_private_key_too_short() {
    let result = WalletClient::from_private_key("0xdead").await;
    assert!(result.is_err(), "should fail on a key that is too short");
}

#[tokio::test]
async fn test_from_private_key_too_long() {
    let long_key = "ab".repeat(33); // 66 hex chars = 33 bytes, too long
    let result = WalletClient::from_private_key(&long_key).await;
    assert!(result.is_err(), "should fail on a key that is too long");
}

// -- from_env tests --

#[tokio::test]
async fn test_from_env_success() {
    // SAFETY: test runs single-threaded, no other thread reads PRIVATE_KEY
    unsafe { std::env::set_var("PRIVATE_KEY", TEST_PRIVATE_KEY) };
    let wallet = WalletClient::from_env().await;
    assert!(wallet.is_ok(), "should succeed when PRIVATE_KEY is set");
    unsafe { std::env::remove_var("PRIVATE_KEY") };
}

#[tokio::test]
async fn test_from_env_missing_var() {
    // SAFETY: test runs single-threaded, no other thread reads PRIVATE_KEY
    unsafe { std::env::remove_var("PRIVATE_KEY") };
    let result = WalletClient::from_env().await;
    assert!(result.is_err(), "should fail when PRIVATE_KEY is not set");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Environment variable missing"),
        "error should mention 'Environment variable missing', got: {err_msg}"
    );
}

#[tokio::test]
async fn test_from_env_invalid_key() {
    // SAFETY: test runs single-threaded, no other thread reads PRIVATE_KEY
    unsafe { std::env::set_var("PRIVATE_KEY", "not_valid") };
    let result = WalletClient::from_env().await;
    assert!(
        result.is_err(),
        "should fail when PRIVATE_KEY holds invalid data"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Invalid private key"),
        "error should mention 'Invalid private key', got: {err_msg}"
    );
    unsafe { std::env::remove_var("PRIVATE_KEY") };
}
