use crate::types::constants::Constants;
use std::fs::{create_dir_all, read_dir, rename};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use alloy::primitives::{Address, hex};
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::signers::local::LocalSigner;
use eyre::{Result, ensure};
use rand::thread_rng;
use rpassword::read_password;

fn prompt_hidden(msg: &str) -> Result<String> {
    print!("{msg}");
    io::stdout().flush()?;
    Ok(read_password()?)
}

fn parse_private_key(input: &str) -> Result<[u8; 32]> {
    let trimmed: &str = input.trim();
    let pk_hex: &str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    hex::decode(pk_hex)?
        .try_into()
        .map_err(|_| eyre::eyre!("Private key must be exactly 32 bytes"))
}

pub fn create_keystore() -> Result<String> {
    let pk_input: String = prompt_hidden("Enter your private key: ")?;
    let private_key: [u8; 32] = parse_private_key(&pk_input)?;

    let password: String = prompt_hidden("Enter password for keystore: ")?;
    let password_confirm: String = prompt_hidden("Confirm password: ")?;

    ensure!(password == password_confirm, "Passwords do not match");

    let output_dir: &Path = Path::new(Constants::KEYSTORE_DIR);
    create_dir_all(output_dir)?;

    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let (wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(output_dir, &mut rng, private_key, &password, None)?;

    let address: Address = wallet.address();
    println!("Derived address: {address}");

    let uuid_path: PathBuf = output_dir.join(&uuid);
    let ks_filename: String = format!("ks-{address}");
    let ks_path: PathBuf = output_dir.join(&ks_filename);

    ensure!(
        !ks_path.exists(),
        "Keystore file already exists: {}",
        ks_path.display()
    );

    rename(&uuid_path, &ks_path)?;

    let recovered: LocalSigner<SigningKey> = LocalSigner::decrypt_keystore(&ks_path, &password)?;
    ensure!(
        address == recovered.address(),
        "Address mismatch after decrypt: expected {address}, got {}",
        recovered.address()
    );

    println!("Keystore saved to {}", ks_path.display());

    Ok(pk_input.trim().to_string())
}

fn prompt_visible(msg: &str) -> Result<String> {
    print!("{msg}");
    io::stdout().flush()?;
    let mut input: String = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn load_keystore() -> Result<String> {
    let dir: &Path = Path::new(Constants::KEYSTORE_DIR);

    let entries: Vec<String> = read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|name| name.starts_with("ks-"))
        .collect();

    if entries.is_empty() {
        return Err(eyre::eyre!("No keystores found"));
    }

    println!("Available keystores:");
    for (i, name) in entries.iter().enumerate() {
        println!("  [{}] {}", i + 1, name.strip_prefix("ks-").unwrap_or(name));
    }
    println!("  [{}] Create new keystore", entries.len() + 1);

    let selection: String = prompt_visible("Select option (number): ")?;
    let idx: usize = selection
        .parse::<usize>()
        .map_err(|_| eyre::eyre!("Invalid number"))?;

    if idx == entries.len() + 1 {
        return create_keystore();
    }

    let ks_name: &String = entries
        .get(idx - 1)
        .ok_or_else(|| eyre::eyre!("Selection out of range"))?;

    let password: String = prompt_hidden("Enter keystore password: ")?;

    let ks_path: PathBuf = dir.join(ks_name);
    let signer: LocalSigner<SigningKey> = LocalSigner::decrypt_keystore(&ks_path, &password)?;

    println!("Loaded keystore: {}", signer.address());

    let key_bytes = signer.credential().to_bytes();
    Ok(hex::encode(key_bytes))
}

pub fn load_or_create_keystore() -> Result<String> {
    let dir: &Path = Path::new(Constants::KEYSTORE_DIR);
    let has_keystores: bool = dir.is_dir()
        && read_dir(dir)?
            .filter_map(Result::ok)
            .any(|e| e.file_name().to_str().is_some_and(|n| n.starts_with("ks-")));

    if has_keystores {
        load_keystore()
    } else {
        create_keystore()
    }
}
