# CLI Reference

## Usage

```
deterministic-deployer-evm [<contract.sol>] [OPTIONS] --chain1 [--chain2 ...]
```

## Positional Argument

| Argument | Description |
|----------|-------------|
| `<contract.sol>` | Path to Solidity source file. Used to read bytecode from Foundry artifacts (`out/`). Optional if using `--contract-name` with a registry entry. |

## Options

| Flag | Value | Description |
|------|-------|-------------|
| `--salt` | `<hex\|uint256>` | CREATE2 salt. Accepts 32-byte hex (`0x00...e90d2d90`) or decimal. Required for artifact path. |
| `--contract-name` | `<name>` | Contract name. Used for registry lookup or to select a specific contract from a multi-contract `.sol` file. |
| `--address` | `<hex>` | Expected deployed address. Used for registry lookup. With or without `0x` prefix. |
| `--constructor-args` | `<hex>` | ABI-encoded constructor arguments. Appended to creation bytecode. With or without `0x` prefix. |
| `--verify` | | Enable Etherscan source verification after deployment. Requires `ETHERSCAN_API_KEY` env var. |
| `--keystore` | | Use encrypted keystore instead of `PRIVATE_KEY` env var. Prompts interactively. |
| `-h, --help` | | Print usage and exit. |

## Chain Flags

At least one chain must be specified.

### Mainnets

| Flag | Chain |
|------|-------|
| `--ethereum` | Ethereum |
| `--base` | Base |
| `--arbitrum` | Arbitrum One |
| `--bnb` | BNB Smart Chain |
| `--avalanche` | Avalanche C-Chain |
| `--polygon` | Polygon |
| `--sonic` | Sonic |
| `--optimism` | Optimism |
| `--zora` | Zora |
| `--arbitrum-nova` | Arbitrum Nova |
| `--polygon-zkevm` | Polygon zkEVM |
| `--gnosis` | Gnosis |
| `--scroll` | Scroll |
| `--linea` | Linea |
| `--plasma` | Plasma |
| `--mantle` | Mantle |
| `--monad` | Monad |
| `--unichain` | Unichain |
| `--celo` | Celo |
| `--zksync` | zkSync Era |
| `--soneium` | Soneium |

### Testnets

| Flag | Chain |
|------|-------|
| `--sepolia` | Ethereum Sepolia |
| `--base-sepolia` | Base Sepolia |
| `--arbitrum-sepolia` | Arbitrum Sepolia |
| `--optimism-sepolia` | Optimism Sepolia |

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `PRIVATE_KEY` | Yes (unless `--keystore`) | Deployer private key (hex, with or without `0x`) |
| `ETHERSCAN_API_KEY` | Only with `--verify` | Etherscan V2 API key (works across all chains) |
| `SOLC_VERSION` | Only with `--verify` + JSON path | Solidity compiler version (e.g., `v0.8.28+commit.7893614a`) |

## Examples

```bash
# Registry deploy to single testnet
deterministic-deployer-evm --contract-name ERC20Mock --sepolia

# Artifact deploy with constructor args
forge build
deterministic-deployer-evm src/MyToken.sol \
  --salt 0x1234 --constructor-args 0xabcd --sepolia

# Multi-chain deploy + verify
ETHERSCAN_API_KEY=xxx deterministic-deployer-evm \
  --contract-name ERC20Mock \
  --ethereum --base --arbitrum --optimism \
  --verify

# Use keystore instead of .env
deterministic-deployer-evm --contract-name ERC20Mock --sepolia --keystore

# Already deployed? Just verify
deterministic-deployer-evm --contract-name ERC20Mock \
  --sepolia --base-sepolia \
  --verify
```
