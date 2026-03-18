use criterion::{Criterion, criterion_group, criterion_main};
use deterministic_deployer_evm::client::wallet_client::WalletClient;

const TEST_PRIVATE_KEY: &str =
    "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

fn bench_wallet_init_single(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("wallet_init_single", |b| {
        b.iter(|| rt.block_on(WalletClient::from_private_key(TEST_PRIVATE_KEY)))
    });
}

fn bench_wallet_init_100_sequential(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("wallet_init_100_sequential", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..100 {
                    let _ = WalletClient::from_private_key(TEST_PRIVATE_KEY).await;
                }
            })
        })
    });
}

fn bench_wallet_init_100_parallel(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("wallet_init_100_parallel", |b| {
        b.iter(|| {
            rt.block_on(async {
                let futures: Vec<_> = (0..100)
                    .map(|_| WalletClient::from_private_key(TEST_PRIVATE_KEY))
                    .collect();
                futures::future::join_all(futures).await
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
┌────────────────────────────────┬──────────┬──────────┬──────────┐
│          Benchmark             │   Low    │   Mean   │   High   │                              
├────────────────────────────────┼──────────┼──────────┼──────────┤                            
│ wallet_init_single             │ 20.85 µs │ 20.96 µs │ 21.10 µs │                              
├────────────────────────────────┼──────────┼──────────┼──────────┤                              
│ wallet_init_100_sequential     │  2.08 ms │  2.09 ms │  2.09 ms │                              
├────────────────────────────────┼──────────┼──────────┼──────────┤                              
│ wallet_init_100_parallel       │  2.08 ms │  2.09 ms │  2.11 ms │                              
└────────────────────────────────┴──────────┴──────────┴──────────┘ 
*/