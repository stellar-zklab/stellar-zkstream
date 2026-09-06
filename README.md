# stellar-zkstream 🌊🔐

![Soroban](https://img.shields.io/badge/Soroban-Protocol_25-blue?style=flat&logo=stellar)
![License](https://img.shields.io/badge/License-Apache_2.0-green)
[![CI](https://github.com/stellar-zklab/stellar-zkstream/actions/workflows/ci.yml/badge.svg)](https://github.com/stellar-zklab/stellar-zkstream/actions/workflows/ci.yml)
![ZKP](https://img.shields.io/badge/Proof-Groth16_BN254-purple)
[![Live Demo](https://img.shields.io/badge/Live_Demo-stellar--zkstream.vercel.app-black?style=flat&logo=vercel)](https://stellar-zkstream.vercel.app/)

Privacy-Preserving Continuous Payment Streaming Protocol on Soroban (Groth16 ZK Range Proofs & Protocol 25 BN254 Host Functions).

**[🔗 Try the live demo](https://stellar-zkstream.vercel.app/)** — wired to the real deployed testnet contracts listed below, not a mockup.

## Why this is a real ZK protocol, not a demo

- **The origin of this ecosystem's real BN254 verifier.** This repo's `zk_verifier` contract was the first place a genuine Groth16 pairing check was proven to work against Soroban Protocol 25's native `env.crypto().bn254()` host functions — `stellar-zkident` later reused this exact contract unmodified for its own three verifier deployments.
- **Real range and nullifier proofs, not opaque flags.** `create_stream` is gated by an actual on-chain Groth16 range-proof verification (proving a stream's amount falls within a committed range without revealing it), and `withdraw` by a real nullifier-proof check that also prevents double-spending a claim.
- **A real edge case found and fixed during testing.** `groth16::is_zero` explicitly rejects degenerate point-at-infinity inputs — a genuine cryptographic footgun this project actually hit and documented, not a hypothetical.
- **Confirmed live, not just unit-tested.** The deployed `range_proof` verifier has been invoked directly on testnet with the project's real proof/public-input files and returned `true` — see Deployment below.

## Architecture

```
+----------------------------+
|           Sender           |
+----------------------------+
            |  create_stream(range proof)
            v
+----------------------------+
|           stream           |
|      (escrow + linear      |
|     vesting w/ cliff)      |
+----------------------------+
            |  verify() -- real Groth16 BN254 pairing check
            v
+----------------------------+
|        zk_verifier         |
|       (range_proof)        |
+----------------------------+

+----------------------------+
|         Recipient          |
+----------------------------+
            |  withdraw(nullifier proof)
            v
+----------------------------+
|           stream           |
|      (marks nullifier      |
|      used, pays out)       |
+----------------------------+
            |  verify() -- real Groth16 BN254 pairing check
            v
+----------------------------+
|        zk_verifier         |
|        (nullifier)         |
+----------------------------+
```

Both verifiers perform real Groth16 BN254 pairing checks — no mocked verification on either path.

## Current Status — what's real vs. not

**`contracts/stream` — real.** `create_stream`, `withdraw` (nullifier-gated, correct linear vesting with cliff), `cancel_stream` (correctly splits vested/unvested funds between sender and recipient), and `create_batch_streams` (atomic multi-stream creation) are all implemented and tested. Now holds two verifier addresses (`range_verifier`, `nullifier_verifier`) instead of one — see `circuits/` below for why.

**`contracts/zk_verifier` — real, and now actually initialized with the real circuits' VKs.** `verify()` in `contracts/zk_verifier/src/groth16.rs` performs an actual Groth16 pairing check against Soroban Protocol 25's native BN254 host functions (`env.crypto().bn254()`: `g1_add`, `g1_mul`, `pairing_check`) — this required bumping the project from `soroban-sdk` v22 to v25, since BN254 support doesn't exist before v25. It correctly rejects a proof presented against the wrong public input, and explicitly rejects degenerate point-at-infinity inputs (a real edge case found during testing — see `groth16::is_zero`'s doc comment). Two separate deployments now exist — one per circuit, since `range_proof` and `nullifier` have different VKs and one `zk_verifier` instance only holds one — each initialized with the real VK from an actual Groth16 trusted-setup pipeline run against the real circuits (see `circuits/` below), not the toy demo circuit this was originally verified with. `stream`'s `create_stream`/`withdraw` call through to the matching one for real.

**Removed: `contracts/token_wrapper` (2026-09-05).** This was a bare `#[contract]` stub with a single `version() -> 1` function, meant to become a SEP-41 allowance-based wrapper. It never shipped, and once `stream` was actually built, it turned out not to be needed at all: `create_stream` funds itself with a plain, sender-authorized `TokenClient::transfer()` against whatever real SEP-41 token address it's given — including native XLM's own Stellar Asset Contract, which is SEP-41 compliant out of the box. Unlike Ethereum, Soroban has no "wrap the native asset" problem to solve, so there was no real gap left for this contract to fill. Removed rather than left as a permanent placeholder, matching `soroban-gasless-contracts`' `gas-estimator` removal for the same reason. It was never deployed, so there's no stale address to keep a record of.

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

## Usage

```typescript
import { StellarZkStreamClient } from '@stellar-zklab/zkstream-sdk';
import freighter from '@stellar/freighter-api';

const zkstream = new StellarZkStreamClient({
  streamContractId: 'CACRWU5VCHIGBMSJZMWDXE3L6UJNJIQ7O4FH32ER3M77AO3Z23562MPH', // live on testnet, see Deployment above
  signTransaction: async (xdr, opts) => {
    const { signedTxXdr } = await freighter.signTransaction(xdr, opts);
    return signedTxXdr;
  },
});

// proof/publicInputs come from a real snarkjs run against circuits/build/range_proof —
// this SDK doesn't generate proofs itself, see the note at the top of sdk/src/client.ts.
const streamId = await zkstream.createStream({
  sender, recipient, token,
  totalAmount: 1_000_0000000n,
  startTime, cliffTime, endTime,
  cancelable: true,
  proof, publicInputs,
});

// claimable_amount() is computed by the contract's own real on-chain vesting math.
const claimable = await zkstream.getClaimableAmount(streamId);
```

See [`sdk/README.md`](sdk/README.md) for the full API and [`circuits/README.md`](circuits/README.md) for how to generate a real range or nullifier proof.

## Ecosystem

Part of **stellar-zklab**'s Soroban Protocol 25 project suite, alongside:
- [`soroban-yield-vault`](https://github.com/stellar-zklab/soroban-yield-vault) — real Blend Protocol V2 yield vault with Yearn V3 share math ([live demo](https://soroban-yield-vault.vercel.app/))
- [`stellar-zkident`](https://github.com/stellar-zklab/stellar-zkident) — self-sovereign DID + real Groth16 credentials, reusing this repo's `zk_verifier` contract unmodified ([live demo](https://stellar-zkident.vercel.app/))

All three share the same "real vs. not" documentation discipline and the same Protocol 25 BN254/testnet deployment conventions.

## 🚀 Quick Start

**Prerequisites**: Rust with the `wasm32v1-none` target, Node.js 20+, and (only for regenerating circuits) `circom` + `snarkjs` — see [`circuits/README.md`](circuits/README.md).

```bash
# Run the real contract test suite (5 for stream, 10 for zk_verifier)
cargo test --all --features testutils

# Run the frontend against the real deployed contracts (connects Freighter, real stream calls)
cd frontend && npm install && npm run dev
```
