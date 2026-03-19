use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{Address, U256, address};
use deterministic_deployer_evm::client::public_client::PublicClient;

// ── Constructor Tests ───────────────────────────────────

#[tokio::test]
async fn test_new_mainnet_ethereum() {
    let client = PublicClient::new_public_provider("mainnet", "ethereum").await;
    assert!(client.is_ok(), "should connect to mainnet ethereum");
    let client = client.unwrap();
    let chain_id = client.get_chain_id().await.unwrap();
    assert_eq!(chain_id, 1, "ethereum mainnet chain_id should be 1");
}

#[tokio::test]
async fn test_new_mainnet_base() {
    let client = PublicClient::new_public_provider("mainnet", "base").await;
    assert!(client.is_ok(), "should connect to mainnet base");
    let client = client.unwrap();
    let chain_id = client.get_chain_id().await.unwrap();
    assert_eq!(chain_id, 8453, "base mainnet chain_id should be 8453");
}

#[tokio::test]
async fn test_new_testnet_sepolia() {
    let client = PublicClient::new_public_provider("testnet", "sepolia").await;
    assert!(client.is_ok(), "should connect to testnet sepolia");
    let client = client.unwrap();
    let chain_id = client.get_chain_id().await.unwrap();
    assert_eq!(
        chain_id, 11155111,
        "sepolia chain_id should be 11155111"
    );
}

#[tokio::test]
async fn test_new_invalid_chain() {
    let result = PublicClient::new_public_provider("mainnet", "nonexistent").await;
    assert!(result.is_err(), "should fail for unknown chain");
}

#[tokio::test]
async fn test_new_invalid_network() {
    let result = PublicClient::new_public_provider("devnet", "ethereum").await;
    assert!(result.is_err(), "should fail for unknown network");
}

#[tokio::test]
async fn test_from_url() {
    let client = PublicClient::new_public_provider_from_url("https://ethereum-rpc.publicnode.com");
    assert!(client.is_ok(), "should create client from raw URL");
    let client = client.unwrap();
    let chain_id = client.get_chain_id().await.unwrap();
    assert_eq!(chain_id, 1, "should connect to ethereum mainnet");
}

// ── Provider Method Tests ───────────────────────────────

#[tokio::test]
async fn test_get_block_number() {
    let client = PublicClient::new_public_provider_from_url("https://ethereum-rpc.publicnode.com").unwrap();
    let block = client.get_block_number().await.unwrap();
    assert!(block > 0, "block number should be > 0, got {block}");
}

#[tokio::test]
async fn test_get_balance() {
    let client = PublicClient::new_public_provider_from_url("https://ethereum-rpc.publicnode.com").unwrap();
    let balance = client.get_balance(Address::ZERO).await.unwrap();
    // Zero address has some balance from dust, just check it doesn't error
    assert!(balance >= U256::ZERO, "balance should be >= 0");
}

#[tokio::test]
async fn test_get_gas_price() {
    let client = PublicClient::new_public_provider_from_url("https://ethereum-rpc.publicnode.com").unwrap();
    let gas_price = client.get_gas_price().await.unwrap();
    assert!(gas_price > 0, "gas price should be > 0, got {gas_price}");
}

#[tokio::test]
async fn test_get_block_by_number() {
    let client = PublicClient::new_public_provider_from_url("https://ethereum-rpc.publicnode.com").unwrap();
    let block = client
        .get_block_by_number(BlockNumberOrTag::Number(1))
        .await
        .unwrap();
    assert!(block.is_some(), "block 1 should exist on ethereum");
}

#[tokio::test]
async fn test_get_code_eoa() {
    let client = PublicClient::new_public_provider_from_url("https://ethereum-rpc.publicnode.com").unwrap();
    let code = client.get_code(Address::ZERO).await.unwrap();
    assert!(code.is_empty(), "EOA should have empty code");
}

#[tokio::test]
async fn test_get_code_contract() {
    let client = PublicClient::new_public_provider_from_url("https://ethereum-rpc.publicnode.com").unwrap();
    // WETH on mainnet
    let weth: Address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    let code = client.get_code(weth).await.unwrap();
    assert!(!code.is_empty(), "WETH contract should have non-empty code");
}

#[tokio::test]
async fn test_accessors() {
    let client = PublicClient::new_public_provider("mainnet", "ethereum").await.unwrap();
    assert_eq!(client.chain(), "ethereum");
    assert_eq!(client.network(), "mainnet");
    assert_eq!(client.rpc_url(), "https://ethereum-rpc.publicnode.com");
}
