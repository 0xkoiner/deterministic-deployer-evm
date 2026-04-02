# Architecture

## Directory Structure

```rust
src
├── bin
│   ├── all.rs
│   ├── create_keystore.rs
│   └── public_client.rs
├── client
│   ├── mod.rs
│   ├── public_client.rs. // PublicClient: DynProvider wrapper with RPC methods
│   └── wallet_client.rs  // WalletClient: signer + signed provider per chain
├── data
│   ├── contracts
│   │   ├── contract.rs. // ContractSpec struct + runtime builder
│   │   ├── mod.rs
│   │   └── registry.rs  // Static contract registry (CONTRACTS array)
│   ├── explorers_config.toml  // Block explorer URLs per chain (embedded at compile time)
│   ├── keystore
│   │   └── ks-address
│   ├── mod.rs
│   └── rpc_config.toml  // RPC endpoints per chain (embedded at compile time)
├── helpers
│   ├── balance_checker.rs  // Check deployer balance > 0
│   ├── code_checker.rs  // Check if bytecode exists at address
│   ├── contract_searcher.rs  // Resolve contract by name/address/path
│   ├── mod.rs
│   └── pre_conditions.rs. // Pre-check phase: has_code, logging, CREATE2 verification
├── lib.rs  // Module exports
├── main.rs // Entry point, CLI orchestration
├── types
│   ├── config.rs  // RpcConfig, ExplorerConfig (TOML deserialization targets)
│   ├── constants.rs // Static constants, embedded TOML, env var names
│   ├── errors.rs // All error enums (thiserror)
│   └── mod.rs
└── utils
    ├── artifact.rs // Read creation bytecode from Foundry artifacts
    ├── create_2.rs // CREATE2 address computation and verification
    ├── create_keystore.rs  // Keystore creation, loading, selection
    ├── deploy.rs  // Transaction building, sending, receipt handling
    ├── init_explorers.rs  // OnceLock-based explorer config loader
    ├── init_rpc.rs  // OnceLock-based RPC config loader
    ├── mod.rs
    ├── read_buf.rs  // CLI argument parsing (lexopt)
    └── verifier.rs  // Etherscan verification (forge + API)

9 directories, 35 files
```

## Data Flow

```rust
CLI args (parse_args)
     |
     v
resolve_contract ──> Registry (find_by_name/address/path)
     |                    |
     |  not found         | found
     v                    v
create_contract_spec_from_args    ContractSpec (static)
  |  (read artifact, Box::leak)
  v
ContractSpec (runtime)
     |
     +-----> check_before (CREATE2 address verification)
     |
     +-----> create_deployers (WalletClient per chain)
     |
     +-----> run_prechecks (parallel: has_code x2 per chain)
     |            |
     |    needs_deploy    ready_for_verify
     |         |                  |
     +-----> run_deployments      |
     |    (parallel: balance +    |
     |     deploy + confirm)      |
     |         |                  |
     |    deployed wallets ------>+
     |                            |
     +-----> run_verifications (parallel, staggered)
              (forge verify-contract --watch)
```

## Key Types

### ContractSpec

```rust
pub struct ContractSpec {
    pub name: &'static str,
    pub address: Option<Address>,
    pub salt: Option<B256>,
    pub path: Option<&'static str>,
    pub deployer_tx: Option<&'static [u8]>,
    pub constructor_args: Option<&'static [u8]>,
    pub creation_bytecode: Option<&'static [u8]>,
    pub verify_json_path: Option<&'static str>,
}
```

All fields are `&'static` for zero-cost `Copy`. Runtime specs use `Box::leak` to promote heap data to static lifetime (valid for CLI process lifetime).

### WalletClient

```rust
pub struct WalletClient {
    signer: PrivateKeySigner,
    public: Option<PublicClient>,
}
```

One per chain. The `PublicClient` holds a signed `DynProvider` that handles both reads and signed transactions through a single HTTP connection.

### PublicClient

```rust
pub struct PublicClient {
    provider: DynProvider,
    chain: &'static str,
    network: &'static str,
    rpc_url: Cow<'static, str>,
}
```

The `DynProvider` is created with `ProviderBuilder::new().wallet(signer).connect_http(url).erased()`. The wallet filler layer is preserved through erasure, enabling `send_transaction` on the trait object.

## Config Loading

Both RPC and explorer configs use the same pattern:

1. TOML embedded at compile time via `include_str!`
2. Parsed once on first access via `OnceLock`
3. Returned as `&'static Config` on subsequent calls
4. Lookup by `(network, chain)` key pair

No file I/O at runtime. No repeated parsing.

## Error Handling

Every module has its own error enum in `src/types/errors.rs`, using `thiserror` for `Display` + `Error` derives. Errors propagate via `Result<T, E>` and `?` operator. No `unwrap()` in production paths.

| Error Type | Used By |
|------------|---------|
| `CliError` | `read_buf.rs` (argument parsing) |
| `RpcError` | `init_rpc.rs` (config loading) |
| `ExplorerError` | `init_explorers.rs` (config loading) |
| `WalletError` | `wallet_client.rs` (signer creation) |
| `PublicClientError` | `public_client.rs` (provider setup) |
| `Create2Error` | `create_2.rs` (address computation) |
| `DeployError` | `deploy.rs` (transaction lifecycle) |
| `VerifierError` | `verifier.rs` (Etherscan API) |
| `ArtifactError` | `artifact.rs` (bytecode reading) |
| `BalanceCheckerError` | `balance_checker.rs` |
| `CodeCheckerError` | `code_checker.rs` |
