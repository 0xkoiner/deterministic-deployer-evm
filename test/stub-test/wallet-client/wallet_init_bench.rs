use criterion::{Criterion, criterion_group, criterion_main};
use deterministic_deployer_evm::client::wallet_client::WalletClient;

const TEST_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

fn bench_wallet_init_single(c: &mut Criterion) {
    c.bench_function("wallet_init_single", |b| {
        b.iter(|| WalletClient::from_private_key(TEST_PRIVATE_KEY))
    });
}

fn bench_wallet_init_100_sequential(c: &mut Criterion) {
    c.bench_function("wallet_init_100_sequential", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = WalletClient::from_private_key(TEST_PRIVATE_KEY);
            }
        })
    });
}

fn bench_wallet_init_100_parallel(c: &mut Criterion) {
    c.bench_function("wallet_init_100_parallel", |b| {
        b.iter(|| {
            std::thread::scope(|s| {
                let handles: Vec<_> = (0..100)
                    .map(|_| s.spawn(|| WalletClient::from_private_key(TEST_PRIVATE_KEY)))
                    .collect();
                handles
                    .into_iter()
                    .map(|h| h.join().unwrap())
                    .collect::<Vec<_>>()
            })
        })
    });
}

criterion_group!(
    benches,
    bench_wallet_init_single,
    bench_wallet_init_100_sequential,
    bench_wallet_init_100_parallel
);
criterion_main!(benches);

/*
┌────────────────────────────┬───────────┬───────────┬────────┐
│         Benchmark          │  Before   │   After   │ Change │
├────────────────────────────┼───────────┼───────────┼────────┤
│ wallet_init_single         │ ~20.96 µs │ ~20.82 µs │ ~-0.5% │
├────────────────────────────┼───────────┼───────────┼────────┤
│ wallet_init_100_sequential │ ~2.09 ms  │ ~2.08 ms  │ ~-0.5% │
├────────────────────────────┼───────────┼───────────┼────────┤
│ wallet_init_100_parallel   │ ~2.09 ms  │ ~1.24 ms  │ -40.8% │
└────────────────────────────┴───────────┴───────────┴────────┘
*/
