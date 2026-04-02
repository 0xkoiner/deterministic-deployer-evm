use alloy::eips::BlockNumberOrTag;
use alloy::primitives::Address;
use criterion::{Criterion, criterion_group, criterion_main};
use deterministic_deployer_evm::types::config::PublicClient;
use tokio::runtime::Runtime;

const RPC_URL: &str = "https://ethereum-rpc.publicnode.com";

fn make_client() -> PublicClient {
    PublicClient::new_public_provider_from_url(RPC_URL).unwrap()
}

fn bench_get_chain_id(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = make_client();
    c.bench_function("public_client_get_chain_id", |b| {
        b.to_async(&rt).iter(|| client.get_chain_id())
    });
}

fn bench_get_block_number(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = make_client();
    c.bench_function("public_client_get_block_number", |b| {
        b.to_async(&rt).iter(|| client.get_block_number())
    });
}

fn bench_get_balance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = make_client();
    c.bench_function("public_client_get_balance", |b| {
        b.to_async(&rt).iter(|| client.get_balance(Address::ZERO))
    });
}

fn bench_get_block_by_number(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = make_client();
    c.bench_function("public_client_get_block_by_number", |b| {
        b.to_async(&rt)
            .iter(|| client.get_block_by_number(BlockNumberOrTag::Latest))
    });
}

fn bench_10_sequential_rpc_calls(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = make_client();
    c.bench_function("public_client_10_sequential_rpc_calls", |b| {
        b.to_async(&rt).iter(|| async {
            for _ in 0..10 {
                let _ = client.get_block_number().await;
            }
        })
    });
}

fn bench_10_parallel_rpc_calls(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = make_client();
    c.bench_function("public_client_10_parallel_rpc_calls", |b| {
        b.to_async(&rt).iter(|| async {
            let futs: Vec<_> = (0..10).map(|_| client.get_block_number()).collect();
            futures::future::join_all(futs).await
        })
    });
}

criterion_group!(
    benches,
    bench_get_chain_id,
    bench_get_block_number,
    bench_get_balance,
    bench_get_block_by_number,
    bench_10_sequential_rpc_calls,
    bench_10_parallel_rpc_calls
);
criterion_main!(benches);

/*
┌─────────────────────────┬──────────┐
│        Benchmark        │   Time   │
├─────────────────────────┼──────────┤
│ get_chain_id            │ 53.4 ms  │
├─────────────────────────┼──────────┤
│ get_block_number        │ 52.1 ms  │
├─────────────────────────┼──────────┤
│ get_balance             │ 53.6 ms  │
├─────────────────────────┼──────────┤
│ get_block_by_number     │ 59.7 ms  │
├─────────────────────────┼──────────┤
│ 10_sequential_rpc_calls │ 521.6 ms │
├─────────────────────────┼──────────┤
│ 10_parallel_rpc_calls   │ 57.9 ms  │
└─────────────────────────┴──────────┘
*/
