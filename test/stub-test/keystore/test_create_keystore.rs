use std::fs::{self, create_dir_all, rename};
use std::path::PathBuf;

use alloy::primitives::Address;
use alloy::primitives::hex;
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::signers::local::LocalSigner;
use rand::thread_rng;
use serial_test::serial;

/// Anvil account #0 private key (without 0x prefix).
const VALID_PK_NO_PREFIX: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Anvil account #0 private key (with 0x prefix).
const VALID_PK_WITH_PREFIX: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Expected checksummed address for Anvil account #0.
const EXPECTED_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

/// Password used across tests.
const TEST_PASSWORD: &str = "test_password_123";

/// Returns a unique temp directory for each test to avoid collisions.
fn test_output_dir(test_name: &str) -> PathBuf {
    let dir: PathBuf = std::env::temp_dir()
        .join("deterministic_deployer_tests")
        .join(test_name);
    // Clean up from previous runs
    let _ = fs::remove_dir_all(&dir);
    create_dir_all(&dir).expect("Failed to create test output dir");
    dir
}

// Hex decoding & private key parsing

#[test]
fn test_hex_decode_valid_no_prefix() {
    let pk_hex: &str = VALID_PK_NO_PREFIX;
    let bytes: Vec<u8> = hex::decode(pk_hex).expect("Should decode valid hex");
    assert_eq!(bytes.len(), 32, "Private key must be 32 bytes");
}

#[test]
fn test_hex_decode_valid_with_0x_prefix_stripped() {
    let input: &str = VALID_PK_WITH_PREFIX;
    let pk_hex: &str = input.strip_prefix("0x").unwrap_or(input);
    let bytes: Vec<u8> = hex::decode(pk_hex).expect("Should decode after stripping 0x");
    assert_eq!(bytes.len(), 32, "Private key must be 32 bytes");
}

#[test]
fn test_hex_decode_invalid_characters() {
    let invalid_hex: &str = "ZZZZ_not_hex_at_all_ZZZZ0000000000000000000000000000000000000000";
    let result = hex::decode(invalid_hex);
    assert!(result.is_err(), "Should fail on invalid hex characters");
}

#[test]
fn test_hex_decode_odd_length() {
    // 63 hex chars = odd length, not decodable to whole bytes
    let odd_hex: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff8";
    let result = hex::decode(odd_hex);
    assert!(result.is_err(), "Should fail on odd-length hex string");
}

#[test]
fn test_private_key_too_short() {
    let short_hex: &str = "ac0974bec39a17e36ba4a6b4d238ff94";
    let bytes: Vec<u8> = hex::decode(short_hex).expect("Valid hex, just too short");
    let result: Result<[u8; 32], _> = bytes.try_into();
    assert!(result.is_err(), "16 bytes should not convert to [u8; 32]");
}

#[test]
fn test_private_key_too_long() {
    let long_hex: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80aabbccdd";
    let bytes: Vec<u8> = hex::decode(long_hex).expect("Valid hex, just too long");
    let result: Result<[u8; 32], _> = bytes.try_into();
    assert!(result.is_err(), "36 bytes should not convert to [u8; 32]");
}

#[test]
fn test_private_key_empty_string() {
    let empty: &str = "";
    let bytes: Vec<u8> = hex::decode(empty).expect("Empty hex decodes to empty vec");
    let result: Result<[u8; 32], _> = bytes.try_into();
    assert!(
        result.is_err(),
        "Empty bytes should not convert to [u8; 32]"
    );
}

#[test]
fn test_strip_prefix_when_no_prefix() {
    let input: &str = VALID_PK_NO_PREFIX;
    let stripped: &str = input.strip_prefix("0x").unwrap_or(input);
    assert_eq!(
        stripped, VALID_PK_NO_PREFIX,
        "Should return unchanged when no 0x prefix"
    );
}

#[test]
fn test_strip_prefix_with_0x_uppercase() {
    // "0X" (uppercase X) should NOT be stripped by strip_prefix("0x")
    let input: &str = "0Xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let stripped: &str = input.strip_prefix("0x").unwrap_or(input);
    assert_eq!(stripped, input, "Uppercase 0X should not be stripped");
    // This means 0X-prefixed input will fail hex decode — expected behavior
    let result = hex::decode(stripped);
    assert!(result.is_err(), "0X prefix should cause hex decode failure");
}

// Password validation

#[test]
fn test_passwords_match() {
    let password: &str = "my_secure_password";
    let confirm: &str = "my_secure_password";
    assert_eq!(password, confirm, "Matching passwords should be equal");
}

#[test]
fn test_passwords_do_not_match() {
    let password: &str = "password_one";
    let confirm: &str = "password_two";
    assert_ne!(password, confirm, "Different passwords should not be equal");
}

#[test]
fn test_password_empty() {
    let password: &str = "";
    let confirm: &str = "";
    assert_eq!(password, confirm, "Empty passwords should still match");
}

#[test]
fn test_password_whitespace_sensitivity() {
    let password: &str = "password ";
    let confirm: &str = "password";
    assert_ne!(
        password, confirm,
        "Trailing whitespace should make passwords differ"
    );
}

// Keystore encrypt / decrypt

#[test]
#[serial]
fn test_encrypt_keystore_creates_file() {
    let dir: PathBuf = test_output_dir("encrypt_creates_file");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let private_key: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();

    let (wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt_keystore should succeed");

    let uuid_path: PathBuf = dir.join(&uuid);
    assert!(
        uuid_path.exists(),
        "Keystore file should exist at UUID path"
    );

    let address: Address = wallet.address();
    let expected: Address = EXPECTED_ADDRESS.parse().unwrap();
    assert_eq!(
        address, expected,
        "Derived address should match expected Anvil #0 address"
    );
}

#[test]
#[serial]
fn test_decrypt_keystore_recovers_same_address() {
    let dir: PathBuf = test_output_dir("decrypt_same_address");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let private_key: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();

    let (wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt_keystore should succeed");

    let uuid_path: PathBuf = dir.join(&uuid);
    let recovered: LocalSigner<SigningKey> =
        LocalSigner::decrypt_keystore(&uuid_path, TEST_PASSWORD)
            .expect("decrypt_keystore should succeed");

    assert_eq!(
        wallet.address(),
        recovered.address(),
        "Recovered address must match original"
    );
}

#[test]
#[serial]
fn test_decrypt_keystore_wrong_password_fails() {
    let dir: PathBuf = test_output_dir("decrypt_wrong_password");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let private_key: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();

    let (_wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt_keystore should succeed");

    let uuid_path: PathBuf = dir.join(&uuid);
    let result = LocalSigner::<SigningKey>::decrypt_keystore(&uuid_path, "wrong_password");
    assert!(result.is_err(), "Decrypting with wrong password must fail");
}

#[test]
fn test_decrypt_keystore_nonexistent_file_fails() {
    let fake_path: PathBuf = PathBuf::from("/tmp/nonexistent_keystore_file_12345");
    let result = LocalSigner::<SigningKey>::decrypt_keystore(&fake_path, TEST_PASSWORD);
    assert!(result.is_err(), "Decrypting a nonexistent file must fail");
}

#[test]
#[serial]
fn test_decrypt_keystore_corrupted_file_fails() {
    let dir: PathBuf = test_output_dir("decrypt_corrupted");
    let corrupted_path: PathBuf = dir.join("corrupted_keystore.json");
    fs::write(&corrupted_path, "{ not valid keystore json }").expect("Should write corrupted file");

    let result = LocalSigner::<SigningKey>::decrypt_keystore(&corrupted_path, TEST_PASSWORD);
    assert!(
        result.is_err(),
        "Decrypting a corrupted keystore file must fail"
    );
}

#[test]
#[serial]
fn test_decrypt_keystore_empty_file_fails() {
    let dir: PathBuf = test_output_dir("decrypt_empty");
    let empty_path: PathBuf = dir.join("empty_keystore.json");
    fs::write(&empty_path, "").expect("Should write empty file");

    let result = LocalSigner::<SigningKey>::decrypt_keystore(&empty_path, TEST_PASSWORD);
    assert!(result.is_err(), "Decrypting an empty file must fail");
}

// File rename (UUID -> ks-<address>)

#[test]
#[serial]
fn test_rename_uuid_to_ks_address() {
    let dir: PathBuf = test_output_dir("rename_uuid");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let private_key: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();

    let (wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt_keystore should succeed");

    let uuid_path: PathBuf = dir.join(&uuid);
    let address: Address = wallet.address();
    let ks_filename: String = format!("ks-{address}");
    let ks_path: PathBuf = dir.join(&ks_filename);

    rename(&uuid_path, &ks_path).expect("Rename should succeed");

    assert!(
        !uuid_path.exists(),
        "UUID file should no longer exist after rename"
    );
    assert!(
        ks_path.exists(),
        "ks-<address> file should exist after rename"
    );
    assert!(
        ks_filename.starts_with("ks-0x"),
        "Filename should start with ks-0x, got: {ks_filename}"
    );
}

#[test]
#[serial]
fn test_decrypt_after_rename() {
    let dir: PathBuf = test_output_dir("decrypt_after_rename");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let private_key: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();

    let (wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt_keystore should succeed");

    let uuid_path: PathBuf = dir.join(&uuid);
    let address: Address = wallet.address();
    let ks_path: PathBuf = dir.join(format!("ks-{address}"));
    rename(&uuid_path, &ks_path).expect("Rename should succeed");

    // Decrypt from the renamed path
    let recovered: LocalSigner<SigningKey> = LocalSigner::decrypt_keystore(&ks_path, TEST_PASSWORD)
        .expect("Should decrypt from renamed file");

    assert_eq!(
        address,
        recovered.address(),
        "Address after rename + decrypt must match original"
    );
}

// Directory creation

#[test]
#[serial]
fn test_create_dir_all_creates_nested_dirs() {
    let nested: PathBuf = std::env::temp_dir()
        .join("deterministic_deployer_tests")
        .join("nested")
        .join("deep")
        .join("keystore_dir");
    let _ = fs::remove_dir_all(&nested);

    create_dir_all(&nested).expect("create_dir_all should create nested directories");
    assert!(nested.exists(), "Nested directory should exist");
    assert!(nested.is_dir(), "Path should be a directory");

    // Cleanup
    let _ = fs::remove_dir_all(
        std::env::temp_dir()
            .join("deterministic_deployer_tests")
            .join("nested"),
    );
}

#[test]
#[serial]
fn test_create_dir_all_idempotent() {
    let dir: PathBuf = test_output_dir("idempotent_dir");
    // Call again on already-existing dir — should not fail
    let result = create_dir_all(&dir);
    assert!(
        result.is_ok(),
        "create_dir_all on existing dir should succeed"
    );
}

// Full end-to-end flow (without interactive prompts)

#[test]
#[serial]
fn test_full_keystore_flow_no_prefix() {
    let dir: PathBuf = test_output_dir("full_flow_no_prefix");
    let pk_input: &str = VALID_PK_NO_PREFIX;

    // Step 1: Parse private key
    let pk_hex: &str = pk_input.strip_prefix("0x").unwrap_or(pk_input);
    let private_key: [u8; 32] = hex::decode(pk_hex)
        .expect("Should decode hex")
        .try_into()
        .expect("Should be 32 bytes");

    // Step 2: Encrypt
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let (wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt_keystore should succeed");

    // Step 3: Rename
    let address: Address = wallet.address();
    let uuid_path: PathBuf = dir.join(&uuid);
    let ks_path: PathBuf = dir.join(format!("ks-{address}"));
    rename(&uuid_path, &ks_path).expect("Rename should succeed");

    // Step 4: Verify
    let recovered: LocalSigner<SigningKey> =
        LocalSigner::decrypt_keystore(&ks_path, TEST_PASSWORD).expect("decrypt should succeed");
    assert_eq!(address, recovered.address(), "Addresses must match");

    // Step 5: Check file is valid JSON
    let contents: String = fs::read_to_string(&ks_path).expect("Should read keystore file");
    assert!(contents.starts_with('{'), "Keystore should be JSON");
    assert!(
        contents.contains("crypto"),
        "Keystore JSON should contain 'crypto' field"
    );
}

#[test]
#[serial]
fn test_full_keystore_flow_with_0x_prefix() {
    let dir: PathBuf = test_output_dir("full_flow_with_prefix");
    let pk_input: &str = VALID_PK_WITH_PREFIX;

    let pk_hex: &str = pk_input.strip_prefix("0x").unwrap_or(pk_input);
    let private_key: [u8; 32] = hex::decode(pk_hex)
        .expect("Should decode hex")
        .try_into()
        .expect("Should be 32 bytes");

    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let (wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt_keystore should succeed");

    let address: Address = wallet.address();
    let expected: Address = EXPECTED_ADDRESS.parse().unwrap();
    assert_eq!(
        address, expected,
        "Address from 0x-prefixed key should match expected"
    );

    let uuid_path: PathBuf = dir.join(&uuid);
    let ks_path: PathBuf = dir.join(format!("ks-{address}"));
    rename(&uuid_path, &ks_path).expect("Rename should succeed");

    let recovered: LocalSigner<SigningKey> =
        LocalSigner::decrypt_keystore(&ks_path, TEST_PASSWORD).expect("decrypt should succeed");
    assert_eq!(address, recovered.address(), "Addresses must match");
}

// Edge cases: different private keys produce different addresses

#[test]
#[serial]
fn test_different_private_keys_produce_different_addresses() {
    let dir: PathBuf = test_output_dir("different_keys");
    let mut rng: rand::prelude::ThreadRng = thread_rng();

    // Anvil account #0
    let pk1: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();
    // Anvil account #1
    let pk2: [u8; 32] =
        hex::decode("59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
            .unwrap()
            .try_into()
            .unwrap();

    let (wallet1, _): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, pk1, TEST_PASSWORD, None).unwrap();
    let (wallet2, _): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, pk2, TEST_PASSWORD, None).unwrap();

    assert_ne!(
        wallet1.address(),
        wallet2.address(),
        "Different private keys must produce different addresses"
    );
}

// Edge case: all-zero private key (invalid for secp256k1)

#[test]
fn test_zero_private_key_fails() {
    let dir: PathBuf = test_output_dir("zero_key");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let zero_key: [u8; 32] = [0u8; 32];

    let result =
        LocalSigner::<SigningKey>::encrypt_keystore(&dir, &mut rng, zero_key, TEST_PASSWORD, None);
    assert!(
        result.is_err(),
        "All-zero private key is invalid for secp256k1 and must fail"
    );
}

// Edge case: empty password (should still work — password can be empty)

#[test]
#[serial]
fn test_encrypt_decrypt_with_empty_password() {
    let dir: PathBuf = test_output_dir("empty_password");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let private_key: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();

    let (wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, "", None)
            .expect("Empty password should be allowed");

    let uuid_path: PathBuf = dir.join(&uuid);
    let recovered: LocalSigner<SigningKey> = LocalSigner::decrypt_keystore(&uuid_path, "")
        .expect("Decrypt with same empty password should work");

    assert_eq!(wallet.address(), recovered.address());
}

// Edge case: decrypt with empty password when encrypted with non-empty

#[test]
#[serial]
fn test_decrypt_empty_password_when_encrypted_with_nonempty_fails() {
    let dir: PathBuf = test_output_dir("empty_pw_mismatch");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let private_key: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();

    let (_wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt should succeed");

    let uuid_path: PathBuf = dir.join(&uuid);
    let result = LocalSigner::<SigningKey>::decrypt_keystore(&uuid_path, "");
    assert!(
        result.is_err(),
        "Decrypting with empty password when encrypted with non-empty must fail"
    );
}

// Edge case: keystore JSON structure validation

#[test]
#[serial]
fn test_keystore_file_is_valid_json_with_expected_fields() {
    let dir: PathBuf = test_output_dir("json_structure");
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let private_key: [u8; 32] = hex::decode(VALID_PK_NO_PREFIX).unwrap().try_into().unwrap();

    let (_wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
            .expect("encrypt should succeed");

    let contents: String = fs::read_to_string(dir.join(&uuid)).expect("Should read keystore file");

    // Keystore JSON should contain standard fields
    assert!(
        contents.contains("\"crypto\""),
        "Should contain 'crypto' field"
    );
    assert!(
        contents.contains("\"version\""),
        "Should contain 'version' field"
    );
    assert!(contents.contains("\"id\""), "Should contain 'id' field");
}
