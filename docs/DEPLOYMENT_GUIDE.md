# stellar-zkstream Deployment Guide

Deploys two `zk_verifier` instances (one per circuit — see below) and `stream` to Stellar
testnet. All three pass their real test suite (`cargo test --all --features testutils`, see
the repo README) before this guide is relevant — deployment doesn't substitute for that.

## Prerequisites
- **Stellar CLI**: `cargo install --locked stellar-cli`
- **Rust Wasm target**: `rustup target add wasm32v1-none`
- **Binary optimizer** (optional): `wasm-opt`, from the binaryen toolchain
- **The real verification keys already built**: `circuits/build/range_proof/range_proof_vk.hex`
  and `circuits/build/nullifier/nullifier_vk.hex` must exist — see
  [`circuits/README.md`](../circuits/README.md) for how they were produced (a real
  Groth16 trusted-setup pipeline, not placeholders) and how to reproduce them if missing.

## Network
- **Network**: `testnet`
- **RPC URL**: `https://soroban-testnet.stellar.org:443`
- **Passphrase**: `"Test SDF Network ; September 2015"`

## Deploy

```bash
bash scripts/deploy.sh
```

This generates and friendbot-funds a `deployer` testnet identity if one doesn't already
exist, builds all contracts, and deploys + initializes them:

1. `zk_verifier` (range) — `initialize(admin, verification_key=<real range_proof.circom VK>)`
2. `zk_verifier` (nullifier) — `initialize(admin, verification_key=<real nullifier.circom VK>)`
3. `stream` — `initialize(admin, range_verifier=<#1's real deployed address>, nullifier_verifier=<#2's real deployed address>)`

Why two `zk_verifier` instances: `range_proof.circom` and `nullifier.circom` are different
circuits with different verification keys, and one `zk_verifier` contract only holds one VK
at a time. `stream.create_stream()` calls the range verifier; `stream.withdraw()` calls the
nullifier verifier.

Resulting contract IDs are written to `deployments/testnet.json`.

## Verifying it's real, not just deployed

`circuits/build/range_proof/{proof.json,public.json}` and the equivalent `nullifier/` files
contain one real, valid proof for each circuit (see `circuits/README.md`). You can invoke
either deployed verifier with them directly to see a real proof actually verify on-chain:

```bash
stellar contract invoke \
  --id <RANGE_PROOF_VERIFIER_ID> \
  --source deployer \
  --network testnet \
  -- vrfy_prf --proof <hex from circuits/build/range_proof/range_proof_proof.hex> \
     --public_inputs '[<32-byte hex entries from range_proof_public_inputs.json>]'
```
