use std::fs::{self, create_dir_all, rename};
use std::path::PathBuf;

use alloy::primitives::Address;
use alloy::primitives::hex;
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::signers::local::LocalSigner;
use criterion::{Criterion, criterion_group, criterion_main};
use rand::thread_rng;

/// Anvil account #0 private key.
const TEST_PK: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const TEST_PASSWORD: &str = "bench_password_123";

/// Returns a clean temp directory for benchmarks.
fn bench_dir(name: &str) -> PathBuf {
    let dir: PathBuf = std::env::temp_dir()
        .join("deterministic_deployer_bench")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    create_dir_all(&dir).expect("Failed to create bench dir");
    dir
}

/// Parses the test private key into [u8; 32].
fn parse_private_key() -> [u8; 32] {
    hex::decode(TEST_PK).unwrap().try_into().unwrap()
}

// Hex decoding benchmark

fn bench_hex_decode(c: &mut Criterion) {
    c.bench_function("hex_decode_private_key", |b| {
        b.iter(|| {
            let _bytes: Vec<u8> = hex::decode(TEST_PK).unwrap();
        })
    });
}

fn bench_hex_decode_with_strip_prefix(c: &mut Criterion) {
    let input: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    c.bench_function("hex_decode_with_0x_strip", |b| {
        b.iter(|| {
            let pk_hex: &str = input.strip_prefix("0x").unwrap_or(input);
            let _bytes: Vec<u8> = hex::decode(pk_hex).unwrap();
        })
    });
}

// Encrypt keystore benchmark

fn bench_encrypt_keystore_single(c: &mut Criterion) {
    let private_key: [u8; 32] = parse_private_key();

    c.bench_function("encrypt_keystore_single", |b| {
        b.iter(|| {
            let dir: PathBuf = bench_dir("encrypt_single");
            let mut rng: rand::prelude::ThreadRng = thread_rng();
            let (_wallet, _uuid): (LocalSigner<SigningKey>, String) =
                LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
                    .unwrap();
        })
    });
}

// Decrypt keystore benchmark

fn bench_decrypt_keystore_single(c: &mut Criterion) {
    // Setup: create a keystore file once before benchmarking decrypt
    let dir: PathBuf = bench_dir("decrypt_single");
    let private_key: [u8; 32] = parse_private_key();
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let (_wallet, uuid): (LocalSigner<SigningKey>, String) =
        LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None).unwrap();
    let keystore_path: PathBuf = dir.join(&uuid);

    c.bench_function("decrypt_keystore_single", |b| {
        b.iter(|| {
            let _recovered: LocalSigner<SigningKey> =
                LocalSigner::decrypt_keystore(&keystore_path, TEST_PASSWORD).unwrap();
        })
    });
}

// Encrypt + rename (full write path)

fn bench_encrypt_and_rename(c: &mut Criterion) {
    let private_key: [u8; 32] = parse_private_key();

    c.bench_function("encrypt_and_rename_to_ks_address", |b| {
        b.iter(|| {
            let dir: PathBuf = bench_dir("encrypt_rename");
            let mut rng: rand::prelude::ThreadRng = thread_rng();
            let (wallet, uuid): (LocalSigner<SigningKey>, String) =
                LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
                    .unwrap();
            let address: Address = wallet.address();
            let uuid_path: PathBuf = dir.join(&uuid);
            let ks_path: PathBuf = dir.join(format!("ks-{address}"));
            rename(&uuid_path, &ks_path).unwrap();
        })
    });
}

// Full flow: encrypt + rename + decrypt verify

fn bench_full_keystore_flow(c: &mut Criterion) {
    let private_key: [u8; 32] = parse_private_key();

    c.bench_function("full_keystore_flow", |b| {
        b.iter(|| {
            let dir: PathBuf = bench_dir("full_flow");
            let mut rng: rand::prelude::ThreadRng = thread_rng();

            // Encrypt
            let (wallet, uuid): (LocalSigner<SigningKey>, String) =
                LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
                    .unwrap();

            // Rename
            let address: Address = wallet.address();
            let uuid_path: PathBuf = dir.join(&uuid);
            let ks_path: PathBuf = dir.join(format!("ks-{address}"));
            rename(&uuid_path, &ks_path).unwrap();

            // Decrypt & verify
            let recovered: LocalSigner<SigningKey> =
                LocalSigner::decrypt_keystore(&ks_path, TEST_PASSWORD).unwrap();
            assert_eq!(address, recovered.address());
        })
    });
}

// Sequential: 10 keystores in sequence

fn bench_encrypt_10_sequential(c: &mut Criterion) {
    let private_key: [u8; 32] = parse_private_key();

    c.bench_function("encrypt_keystore_10_sequential", |b| {
        b.iter(|| {
            let dir: PathBuf = bench_dir("encrypt_10_seq");
            let mut rng: rand::prelude::ThreadRng = thread_rng();
            for _ in 0..10 {
                let _ = LocalSigner::<SigningKey>::encrypt_keystore(
                    &dir,
                    &mut rng,
                    private_key,
                    TEST_PASSWORD,
                    None,
                )
                .unwrap();
            }
        })
    });
}

// Parallel: 10 keystores in parallel threads

fn bench_encrypt_10_parallel(c: &mut Criterion) {
    let private_key: [u8; 32] = parse_private_key();

    c.bench_function("encrypt_keystore_10_parallel", |b| {
        b.iter(|| {
            let dir: PathBuf = bench_dir("encrypt_10_par");
            std::thread::scope(|s| {
                let handles: Vec<_> = (0..10)
                    .map(|_| {
                        let dir_ref: &PathBuf = &dir;
                        s.spawn(move || {
                            let mut rng: rand::prelude::ThreadRng = thread_rng();
                            LocalSigner::<SigningKey>::encrypt_keystore(
                                dir_ref,
                                &mut rng,
                                private_key,
                                TEST_PASSWORD,
                                None,
                            )
                            .unwrap()
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|h| h.join().unwrap())
                    .collect::<Vec<_>>()
            })
        })
    });
}

// Decrypt 10 sequential

fn bench_decrypt_10_sequential(c: &mut Criterion) {
    // Setup: create 10 keystore files
    let dir: PathBuf = bench_dir("decrypt_10_seq");
    let private_key: [u8; 32] = parse_private_key();
    let mut rng: rand::prelude::ThreadRng = thread_rng();
    let paths: Vec<PathBuf> = (0..10)
        .map(|_| {
            let (_, uuid): (LocalSigner<SigningKey>, String) =
                LocalSigner::encrypt_keystore(&dir, &mut rng, private_key, TEST_PASSWORD, None)
                    .unwrap();
            dir.join(uuid)
        })
        .collect();

    c.bench_function("decrypt_keystore_10_sequential", |b| {
        b.iter(|| {
            for path in &paths {
                let _: LocalSigner<SigningKey> =
                    LocalSigner::decrypt_keystore(path, TEST_PASSWORD).unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    bench_hex_decode,
    bench_hex_decode_with_strip_prefix,
    bench_encrypt_keystore_single,
    bench_decrypt_keystore_single,
    bench_encrypt_and_rename,
    bench_full_keystore_flow,
    bench_encrypt_10_sequential,
    bench_encrypt_10_parallel,
    bench_decrypt_10_sequential,
);
criterion_main!(benches);

/*
┌──────────────────────────────────┬────────────┬────────────┬─────────────────────┐
│ Benchmark                        │ Before     │ After      │ Change              │
├──────────────────────────────────┼────────────┼────────────┼─────────────────────┤
│ hex_decode_private_key           │ ~18.08 ns  │ ~19.45 ns  │ +8.2% (noise)       │
│ hex_decode_with_0x_strip         │ ~18.15 ns  │ ~18.54 ns  │ +4.9% (noise)       │
├──────────────────────────────────┼────────────┼────────────┼─────────────────────┤
│ encrypt_keystore_single          │ ~8.73 ms   │ ~8.46 ms   │ -3.0% improved      │
│ decrypt_keystore_single          │ ~8.62 ms   │ ~8.29 ms   │ -3.8% improved      │
│ encrypt_and_rename_to_ks_address │ ~8.63 ms   │ ~8.63 ms   │  0.0% (no change)   │
│ full_keystore_flow (enc+ren+dec) │ ~17.04 ms  │ ~17.23 ms  │ +1.1% (no change)   │
├──────────────────────────────────┼────────────┼────────────┼─────────────────────┤
│ encrypt_keystore_10_sequential   │ ~85.15 ms  │ ~84.48 ms  │ -0.8% (no change)   │
│ encrypt_keystore_10_parallel     │ ~14.65 ms  │ ~14.66 ms  │  0.0% (no change)   │
│ decrypt_keystore_10_sequential   │ ~83.72 ms  │ ~82.02 ms  │ -2.0% improved      │
└──────────────────────────────────┴────────────┴────────────┴─────────────────────┘
*/
