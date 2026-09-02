# stellar-zkstream 🌊🔐

![Soroban](https://img.shields.io/badge/Soroban-Protocol_25-blue?style=flat&logo=stellar)
![License](https://img.shields.io/badge/License-Apache_2.0-green)
![Build](https://img.shields.io/badge/Cargo_Test-Passing-brightgreen)
![ZKP](https://img.shields.io/badge/Proof-Groth16_BN254-purple)

Privacy-Preserving Continuous Payment Streaming Protocol on Soroban (Groth16 ZK Range Proofs & Protocol 25 BN254 Host Functions).

## Current Status — what's real vs. not

**`contracts/stream` — real.** `create_stream`, `withdraw` (nullifier-gated, correct linear vesting with cliff), `cancel_stream` (correctly splits vested/unvested funds between sender and recipient), and `create_batch_streams` (atomic multi-stream creation) are all implemented and tested.

**`contracts/zk_verifier` — real.** `verify()` in `contracts/zk_verifier/src/groth16.rs` performs an actual Groth16 pairing check against Soroban Protocol 25's native BN254 host functions (`env.crypto().bn254()`: `g1_add`, `g1_mul`, `pairing_check`) — this required bumping the project from `soroban-sdk` v22 to v25, since BN254 support doesn't exist before v25. Verified with a genuine Groth16 proof generated via `arkworks` in the test suite (not a mock): the contract correctly accepts a real valid proof, correctly rejects a proof presented against the wrong public input, and explicitly rejects degenerate point-at-infinity inputs (a real edge case found during testing — see `groth16::is_zero`'s doc comment). `stream`'s `withdraw`/`cancel_stream` calls through to this for real now.

**`contracts/token_wrapper` — not implemented.** A bare `#[contract]` with a single `version() -> 1` function; the SEP-41 allowance-based wrapper described in its own doc comment doesn't exist yet. Not deployed (see Deployment below).

**`circuits/` — real Circom source, not yet compiled into this project's actual verifying key.** `range_proof.circom` and `nullifier.circom` exist and describe the real circuits this protocol needs, but nobody has run them through a trusted setup yet — there's no compiled verifying key from *these* circuits checked in or loaded anywhere. `zk_verifier` itself is a genuine, working Groth16 verifier for any valid proof/VK pair (see above); it just isn't yet initialized with a VK that actually corresponds to `range_proof`/`nullifier`.

## Deployment

`scripts/deploy.sh` deploys both contracts to Stellar testnet and initializes `stream` with
`zk_verifier`'s real deployed address — see
[`docs/DEPLOYMENT_GUIDE.md`](docs/DEPLOYMENT_GUIDE.md). `zk_verifier` itself is deliberately
left un-initialized (see that guide for why); resulting contract IDs land in
`deployments/testnet.json`.

## 🚀 Quick Start
```bash
cargo test --all --features testutils
cd frontend && npm run dev
```
