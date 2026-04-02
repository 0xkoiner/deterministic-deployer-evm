# Dependencies

## Runtime

| Crate | Version | Purpose |
|-------|---------|---------|
| `alloy` | 1.0 | EVM types, providers, signers, transaction building. Features: `full`, `signer-keystore`. |
| `tokio` | 1 | Async runtime. Features: `rt-multi-thread`, `macros`. Powers parallel chain operations. |
| `lexopt` | 0.3 | Minimal CLI argument parser. Zero dependencies, no proc macros. |
| `thiserror` | 2 | Derive `Error` + `Display` for custom error enums. |
| `serde` | 1.0 | Serialization framework. Features: `derive`. Used for TOML and JSON parsing. |
| `serde_json` | 1.0 | JSON parser. Reads Foundry artifact files (`out/*.json`). |
| `toml` | 1.0 | TOML parser. Reads RPC and explorer config files. |
| `log` | 0.4 | Logging facade. Macros: `info!`, `warn!`, `error!`, `debug!`. |
| `env_logger` | 0.11 | Log output backend. Configured with `RUST_LOG` env var. Custom colored formatter. |
| `dotenv` | 0.15 | Loads `.env` file into environment variables. |
| `eyre` | 0.6 | Error handling for keystore operations. Provides `Result`, `ensure!`. |
| `rand` | 0.8 | Random number generation for keystore encryption. |
| `rpassword` | 5 | Hidden terminal input for passwords and private keys. |

## Dev / Test

| Crate | Version | Purpose |
|-------|---------|---------|
| `criterion` | 0.5 | Benchmarking framework. Features: `async_tokio`. |
| `futures` | 0.3 | Async utilities for tests. |
| `serial_test` | 3 | Run tests sequentially when needed. |

## Why These Choices

**alloy over ethers-rs**: alloy is the modern Rust EVM library (by the same team), with better types, native `DynProvider` erasure, and active maintenance. ethers-rs is deprecated.

**lexopt over clap**: The CLI has ~10 flags and 25 chain selectors. lexopt is 0-dependency and compiles in <1s. clap would add ~30 dependencies and ~5s to compile time for no benefit at this scale.

**thiserror over anyhow**: Each module has structured error types with specific variants. thiserror generates `Display` + `Error` impls from enums. anyhow is for unstructured errors in application code.

**eyre (only in keystore)**: The keystore module uses alloy's `LocalSigner` APIs which return `eyre::Result`. Kept local to that module rather than converting everywhere.
