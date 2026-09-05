# stellar-zkstream 🌊🔐

![Soroban](https://img.shields.io/badge/Soroban-Protocol_25-blue?style=flat&logo=stellar)
![License](https://img.shields.io/badge/License-Apache_2.0-green)
![Build](https://img.shields.io/badge/Cargo_Test-Passing-brightgreen)
![ZKP](https://img.shields.io/badge/Proof-Groth16_BN254-purple)

Privacy-Preserving Continuous Payment Streaming Protocol on Soroban (Groth16 ZK Range Proofs & Protocol 25 BN254 Host Functions).

## Current Status — what's real vs. not

**`contracts/stream` — real.** `create_stream`, `withdraw` (nullifier-gated, correct linear vesting with cliff), `cancel_stream` (correctly splits vested/unvested funds between sender and recipient), and `create_batch_streams` (atomic multi-stream creation) are all implemented and tested. Now holds two verifier addresses (`range_verifier`, `nullifier_verifier`) instead of one — see `circuits/` below for why.

**`contracts/zk_verifier` — real, and now actually initialized with the real circuits' VKs.** `verify()` in `contracts/zk_verifier/src/groth16.rs` performs an actual Groth16 pairing check against Soroban Protocol 25's native BN254 host functions (`env.crypto().bn254()`: `g1_add`, `g1_mul`, `pairing_check`) — this required bumping the project from `soroban-sdk` v22 to v25, since BN254 support doesn't exist before v25. It correctly rejects a proof presented against the wrong public input, and explicitly rejects degenerate point-at-infinity inputs (a real edge case found during testing — see `groth16::is_zero`'s doc comment). Two separate deployments now exist — one per circuit, since `range_proof` and `nullifier` have different VKs and one `zk_verifier` instance only holds one — each initialized with the real VK from an actual Groth16 trusted-setup pipeline run against the real circuits (see `circuits/` below), not the toy demo circuit this was originally verified with. `stream`'s `create_stream`/`withdraw` call through to the matching one for real.

**`contracts/token_wrapper` — not implemented.** A bare `#[contract]` with a single `version() -> 1` function; the SEP-41 allowance-based wrapper described in its own doc comment doesn't exist yet. Not deployed (see Deployment below).

**`circuits/` — real, and now actually compiled into this project's real verifying keys.** `range_proof.circom` and `nullifier.circom` have been run through an actual Groth16 trusted-setup pipeline (Powers of Tau + circuit-specific phase 2 + a real contribution each — see [`circuits/README.md`](circuits/README.md) for the full reproducible steps and an important caveat: this is a genuine but single-contributor setup, not a production multi-party ceremony). The resulting VKs are what the two `zk_verifier` deployments below are actually initialized with. `contracts/zk_verifier/src/test.rs`'s `real_zkstream_circuits` test module feeds a real proof for each real circuit through the actual contract logic and confirms it verifies — not a re-derivation, an independent round-trip check.

## Deployment

All three contracts are live on Stellar testnet (deployed 2026-09-03, see
[`deployments/testnet.json`](deployments/testnet.json) — independently checkable on
[stellar.expert](https://stellar.expert/explorer/testnet)):

| Contract | Address |
|---|---|
| `zk_verifier` (range_proof) | `CARWCSIHZ7HCXDCCLRN2JX7SYDAKMZXI53M6AGUUXPRLLT3UJ3WIDLIY` |
| `zk_verifier` (nullifier) | `CALDSWVU2LCI5N56AVSDYCTH7PO6BVT2TFU5WT5XQTJZNZBCOBD2EJR2` |
| `stream` | `CACRWU5VCHIGBMSJZMWDXE3L6UJNJIQ7O4FH32ER3M77AO3Z23562MPH` |

Confirmed live: invoking the deployed range_proof verifier above with the real proof from
`circuits/build/range_proof/{proof.json,public.json}` returns `true` — a genuine Groth16
proof, for the project's actual circuit, verified on real Stellar testnet infrastructure,
not just in a unit test.

`stream` is initialized with both verifiers' real deployed addresses. Both `zk_verifier`
instances are initialized with their circuit's real VK — see
[`docs/DEPLOYMENT_GUIDE.md`](docs/DEPLOYMENT_GUIDE.md). `scripts/deploy.sh` reproduces
this from scratch.

## 🚀 Quick Start
```bash
cargo test --all --features testutils
cd frontend && npm run dev
```
