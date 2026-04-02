# deterministic-deployer-evm

CLI tool for deterministic CREATE2 contract deployment across EVM chains. Deploy the same contract to the same address on every chain, in parallel.

## Features

- Deterministic CREATE2 deployment via `0x4e59b44847b379578588920cA78FbF26c0B4956C`
- Parallel deployment across 25 chains (21 mainnets + 4 testnets)
- Pre-flight simulation via `eth_estimateGas` before broadcasting
- Etherscan source verification via forge or direct API
- Contract registry with pre-computed bytecode and addresses
- Artifact-based deployment from `forge build` output
- Encrypted keystore support for private key management
- Block explorer links for every transaction

## Install

```bash
cargo install --path .
```

## Quick Example

```bash
# Deploy from registry to 4 testnets + verify
PRIVATE_KEY=0x... deterministic-deployer-evm \
  --contract-name ERC20Mock \
  --sepolia --base-sepolia --arbitrum-sepolia --optimism-sepolia \
  --verify

# Deploy from Foundry artifact
forge build
deterministic-deployer-evm \
  lib/openzeppelin-contracts/contracts/mocks/token/ERC20Mock.sol \
  --salt 0xe90d2d90 --sepolia
```

See [documentation/](documentation/) for full reference.

## License

MIT
