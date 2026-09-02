# stellar-zkstream Deployment Guide

Deploys `zk_verifier` and `stream` to Stellar testnet. Both contracts already pass their
real test suite (`cargo test --all --features testutils`, see the repo README) before this
guide is relevant — deployment doesn't substitute for that.

## Prerequisites
- **Stellar CLI**: `cargo install --locked stellar-cli`
- **Rust Wasm target**: `rustup target add wasm32v1-none`
- **Binary optimizer** (optional): `wasm-opt`, from the binaryen toolchain

## Network
- **Network**: `testnet`
- **RPC URL**: `https://soroban-testnet.stellar.org:443`
- **Passphrase**: `"Test SDF Network ; September 2015"`

## Deploy

```bash
bash scripts/deploy.sh
```

This generates and friendbot-funds a `deployer` testnet identity if one doesn't already
exist, builds both contracts, deploys them, initializes `stream` with the real deployed
`zk_verifier` address, and writes the resulting contract IDs to
`deployments/testnet.json`.

## What deploy.sh deliberately does NOT do

`zk_verifier.initialize(admin, verification_key)` is **not** called automatically. A real
Groth16 verification key is required, and the only one this repo can currently produce is
the toy `x*x=y` demo circuit's VK built in `contracts/zk_verifier/src/test.rs` — the actual
`circuits/range_proof.circom` and `circuits/nullifier.circom` circuits aren't compiled into
a VK yet (see the README's Current Status section). Initializing the deployed contract with
a fabricated or empty key would recreate the exact kind of fabrication this project was
rejected for the first time around, so it's left as an explicit, visible gap rather than
silently faked.

To initialize it once a real VK exists:
```bash
stellar contract invoke \
  --id <ZK_VERIFIER_CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- initialize --admin <DEPLOYER_ADDRESS> --verification_key <VK_HEX_BYTES>
```
