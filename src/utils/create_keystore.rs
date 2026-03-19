use crate::types::constants::Constants;
use std::fs::{create_dir_all, rename};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use alloy::primitives::{Address, hex};
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::signers::local::LocalSigner;
use eyre::{Result, ensure};
use rand::thread_rng;
use rpassword::read_password;

fn _prompt_hidden(msg: &str) -> Result<String> {
    print!("{msg}");
    io::stdout().flush()?;
    Ok(read_password()?)
}

fn _parse_private_key(input: &str) -> Result<[u8; 32]> {
    let trimmed: &str = input.trim();
    let pk_hex: &str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    hex::decode(pk_hex)?
        .try_into()
        .map_err(|_| eyre::eyre!("Private key must be exactly 32 bytes"))
}

pub fn create_keystore() -> Result<()> {
    let pk_input: String = _prompt_hidden("Enter your private key: ")?;
    let private_key: [u8; 32] = _parse_private_key(&pk_input)?;

    let password: String = _prompt_hidden("Enter password for keystore: ")?;
    let password_confirm: String = _prompt_hidden("Confirm password: ")?;

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

    Ok(())
}
