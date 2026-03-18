use deterministic_deployer_evm::types::config::RpcConfig;
use deterministic_deployer_evm::utils::init_rpc::{get_rpc, load_config};

#[test]
fn test_load_config_success() {
    let config: RpcConfig = load_config().unwrap();
    assert!(!config.mainnet.is_empty(), "mainnet should have chains");
    assert!(!config.testnet.is_empty(), "testnet should have chains");
}

#[tokio::test]
async fn test_get_rpc_mainnet_ethereum() {
    let config: RpcConfig = load_config().unwrap();
    let url: &str = get_rpc(&config, "mainnet", "ethereum").await.unwrap();
    assert!(
        url.starts_with("https://"),
        "URL should start with https://"
    );
    assert!(url.contains("ethereum"), "URL should contain 'ethereum'");
}

#[tokio::test]
async fn test_get_rpc_testnet_sepolia() {
    let config: RpcConfig = load_config().unwrap();
    let url: &str = get_rpc(&config, "testnet", "sepolia").await.unwrap();
    assert!(
        url.starts_with("https://"),
        "URL should start with https://"
    );
    assert!(url.contains("sepolia"), "URL should contain 'sepolia'");
}

#[tokio::test]
async fn test_revert_unknown_network() {
    let config: RpcConfig = load_config().unwrap();
    let result = get_rpc(&config, "error", "ethereum").await;
    assert!(result.is_err());
    let err_msg: String = result.unwrap_err().to_string();
    assert_eq!(err_msg, "Unknown network: error");
}

#[tokio::test]
async fn test_revert_unknown_chain() {
    let config: RpcConfig = load_config().unwrap();
    let result = get_rpc(&config, "mainnet", "error").await;
    assert!(result.is_err());
    let err_msg: String = result.unwrap_err().to_string();
    assert_eq!(err_msg, "Chain error not found in mainnet");
}
