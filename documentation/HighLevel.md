# How It Works

## Deterministic Deployment

The EVM `CREATE2` opcode computes a contract's address from `deployer + salt + init_code`. Given the same inputs, the address is identical on every chain. This tool sends `salt ++ init_code` as calldata to the deterministic deployer factory at `0x4e59b44847b379578588920cA78FbF26c0B4956C`, which is pre-deployed on all major EVM chains.

*SOURCE: [deterministic-deployment-proxy](https://github.com/Arachnid/deterministic-deployment-proxy)*

```
address = keccak256(0xff ++ deployer ++ salt ++ keccak256(init_code))[12:]
```

## Two Deployment Paths

### Path 1: Contract Registry

Pre-registered contracts with hardcoded bytecode, salt, and expected address. No compilation needed.

```
--contract-name <name>
       |
  Registry lookup (find_by_name)
       |
  ContractSpec { name, salt, creation_bytecode, address, ... }
       |
  Verify CREATE2 address matches
       |
  Deploy
```

### Path 2: Foundry Artifact

Any contract compiled with `forge build`. The tool reads bytecode from `out/`.

```
  lib/MyContract.sol --salt 0x1234 --constructor-args 0xabcd
       |
  Read out/MyContract.sol/MyContract.json
       |
  Extract bytecode.object
       |
  init_code = creation_bytecode + constructor_args
       |
  Compute CREATE2 address
       |
  Build ContractSpec at runtime (Box::leak for &'static)
       |
  Deploy (same flow as registry path)
```

## Execution Phases

### Phase 1: Pre-checks (parallel)

All chains checked concurrently via `JoinSet`:
- Is the deterministic deployer factory deployed on this chain?
- Is the target contract already deployed at the expected address?

Chains are sorted into `needs_deploy` or `ready_for_verify`.

### Phase 2: Deploy (parallel)

For each chain that needs deployment, in one async task per chain:
1. Check deployer balance (skip if zero)
2. `eth_estimateGas` (simulation, catches reverts before broadcasting)
3. `eth_sendTransaction` (signed via the provider's wallet filler)
4. Wait for receipt, check status
5. `eth_getCode` at expected address (confirm deployment)
6. Log explorer link

All chains run concurrently. No sync points between balance check, deploy, and confirmation.

### Phase 3: Verify (parallel, staggered)

If `--verify` is set, all chains verify concurrently with 1-second staggered starts (avoids Etherscan rate limits):

- Uses `forge verify-contract --watch` under the hood
- Forge handles compilation, Standard-Json-Input generation, and Etherscan polling
- Works for both already-deployed and newly-deployed contracts

## Key Management

| Method | How | When |
|--------|-----|------|
| `.env` file | `PRIVATE_KEY=0x...` in `.env` | Default |
| Keystore | `--keystore` flag, interactive prompts | Local development, more secure |

The keystore is encrypted with scrypt KDF and stored in `src/data/keystore/ks-{address}`. On subsequent runs, the tool lists existing keystores for selection.

## Connection Model

Each chain gets a single signed `DynProvider` (alloy's erased provider with wallet filler). This one connection handles:
- All read calls (`eth_getCode`, `eth_getBalance`, `eth_estimateGas`)
- Signed transactions (`eth_sendTransaction`)
- Nonce and chain ID auto-filled by alloy's filler layer

No duplicate connections per chain.
