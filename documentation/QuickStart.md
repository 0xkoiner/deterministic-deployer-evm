# Quick Start

## Prerequisites

- [Rust](https://rustup.rs/) 1.85+
- [Foundry](https://book.getfoundry.sh/getting-started/installation)
- A funded wallet on your target chains
- Etherscan API key

## Install

```bash
git clone https://github.com/0xkoiner/deterministic-deployer-evm
cd deterministic-deployer-evm
cargo install --path .
```

## Setup

### Option A: Private Key via .env

Create a `.env` file in the project root:

```
PRIVATE_KEY=0x..............................
```

### Option B: Encrypted Keystore

Use the `--keystore` flag. On first run, the tool prompts you to create one:

```
$ deterministic-deployer-evm --contract-name <NAME> --sepolia --keystore
Enter your private key:
Enter password for keystore:
Confirm password:
Keystore saved to src/data/keystore/ks-0xA84E...
```

On subsequent runs, it lists existing keystores:

```
Available keystores:
  [1] 0x..............................
  [2] Create new keystore
Select option (number):
```
Set ContractSpec object `src/data/contracts/registry.rs`:
```rust
    ContractSpec {
        name: "MyNewContract",
        address: address!("1234567890abcdef1234567890abcdef12345678"),
        salt: b256!("00000000000000000000000000000000000000000000000000000000deadbeef"),
        path: Some("src/MyContract.sol"),
        deployer_tx: &hex!("6080604052..."),
        constructor_args: Some(&hex!("0000...")),
        creation_bytecode: None,
        verify_json_path: Some("contracts/verify/1234...5678.json"),
    },
```

## Deploy from Registry

Contracts in the built-in registry have pre-computed bytecode and addresses:

```bash
deterministic-deployer-evm --contract-name ERC20Mock --sepolia
```

## Deploy from Foundry Artifact

For any contract compiled with `forge build`:

```bash
forge build
deterministic-deployer-evm \
  lib/openzeppelin-contracts/contracts/mocks/token/ERC4626Mock.sol \
  --salt 0x120ac37d \
  --constructor-args 0x000000000000000000000000000000000000caff \
  --sepolia --base-sepolia
```

The tool reads the creation bytecode from `out/{File}.sol/{Contract}.json`, appends constructor args, computes the CREATE2 address, and deploys.

## Deploy + Verify

Add `--verify` and set the Etherscan API key:

```bash
ETHERSCAN_API_KEY=your_key deterministic-deployer-evm \
  --contract-name ERC20Mock \
  --sepolia --base-sepolia --arbitrum-sepolia --optimism-sepolia \
  --verify
```

Verification runs sequentially with staggered starts after all deployments complete. Uses `forge verify-contract --watch` under the hood.

## Multi-Chain Parallel Deploy

Select multiple chains and they deploy in parallel:

```bash
deterministic-deployer-evm --contract-name ERC20Mock \
  --ethereum --base --arbitrum --optimism --polygon
```

All pre-checks, balance checks, and deployments run concurrently across chains.
