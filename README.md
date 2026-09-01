# stellar-zkstream 🌊🔐

![Soroban](https://img.shields.io/badge/Soroban-Protocol_25-blue?style=flat&logo=stellar)
![License](https://img.shields.io/badge/License-Apache_2.0-green)
![Build](https://img.shields.io/badge/Cargo_Test-Passing-brightgreen)
![ZKP](https://img.shields.io/badge/Proof-Groth16_BN254-purple)

Privacy-Preserving Continuous Payment Streaming Protocol on Soroban (Groth16 ZK Range Proofs & Protocol 25 BN254 Host Functions).

## Current Status — what's real vs. not

**`contracts/stream` — real.** `create_stream`, `withdraw` (nullifier-gated, correct linear vesting with cliff), `cancel_stream` (correctly splits vested/unvested funds between sender and recipient), and `create_batch_streams` (atomic multi-stream creation) are all implemented and tested.

**`contracts/zk_verifier` — structurally real, cryptographically stubbed.** `env.crypto()`'s BN254 pairing host functions genuinely exist as of Soroban Protocol 25 (`soroban-sdk` v25's `crypto/bn254.rs`), so this isn't vaporware — but `verify()` in `contracts/zk_verifier/src/groth16.rs` currently validates proof/VK byte structure only and then **returns `true` unconditionally**. The real pairing-equation check is written out correctly as a comment in that file, just not executed yet. Until that lands, `stream`'s `withdraw`/`cancel_stream` will accept *any* proof bytes — the privacy/ZK guarantee this protocol is named for isn't enforced on-chain yet, even though the streaming mechanics around it are solid.

**`circuits/` — real Circom source, not yet wired to an on-chain verifying key.** `range_proof.circom` and `nullifier.circom` exist; there's no compiled verifying key checked in or referenced by `zk_verifier` yet.

## 🚀 Quick Start
```bash
cargo test --all --features testutils
cd frontend && npm run dev
```
