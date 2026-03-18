use criterion::{Criterion, criterion_group, criterion_main};
use deterministic_deployer_evm::utils::init_rpc::{get_rpc, load_config};

fn bench_load_config(c: &mut Criterion) {
    c.bench_function("load_config", |b| b.iter(|| load_config().unwrap()));
}

fn bench_get_rpc_single(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let config = load_config().unwrap();
    c.bench_function("get_rpc_single", |b| {
        b.iter(|| rt.block_on(get_rpc(&config, "mainnet", "ethereum")))
    });
}

fn bench_get_rpc_100_parallel(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let config = load_config().unwrap();
    c.bench_function("get_rpc_100_parallel", |b| {
        b.iter(|| {
            rt.block_on(async {
                let futures: Vec<_> = (0..100)
                    .map(|_| get_rpc(&config, "mainnet", "ethereum"))
                    .collect();
                futures::future::join_all(futures).await
            })
        })
    });
}

criterion_group!(
    benches,
    bench_load_config,
    bench_get_rpc_single,
    bench_get_rpc_100_parallel
);
criterion_main!(benches);

/*
┌──────────────────────┬─────────┬─────────┬──────────────────────┐
│      Benchmark       │ Before  │  After  │     Improvement      │
├──────────────────────┼─────────┼─────────┼──────────────────────┤
│ load_config          │ 3.65 µs │ 3.71 µs │ ~same (within noise) │
├──────────────────────┼─────────┼─────────┼──────────────────────┤
│ get_rpc_single       │ 54 ns   │ 42 ns   │ -24% faster          │
├──────────────────────┼─────────┼─────────┼──────────────────────┤
│ get_rpc_100_parallel │ 7.47 µs │ 5.65 µs │ -25% faster          │
└──────────────────────┴─────────┴────────────────────────────────┘
*/
