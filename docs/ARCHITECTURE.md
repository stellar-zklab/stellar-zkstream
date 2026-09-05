# Architecture — stellar-zkstream

## Overview

stellar-zkstream is a privacy-preserving payment streaming protocol on Soroban
combining Groth16 ZK proofs (BN254) with on-chain escrow logic.

## Flow

```
Sender → [range_proof circuit] → Groth16 proof → zk_verifier contract → stream contract (escrow)
Recipient → [nullifier circuit] → Groth16 proof → zk_verifier contract → withdraw tokens
```

## Contracts

| Contract       | Role                                        |
|----------------|---------------------------------------------|
| stream         | Stream lifecycle: create, withdraw, cancel  |
| zk_verifier   | Groth16 BN254 proof verifier (Protocol 25)  |

`stream` funds itself directly against any real SEP-41 token via `TokenClient::transfer()` —
there used to be a third contract here, `token_wrapper`, meant to add an allowance-based
wrapper on top of that, but it was removed 2026-09-05 as unnecessary: every real Stellar
asset (native XLM included) is already SEP-41 compliant, so there was no gap left for it to
fill. See the README's "Removed" note.

## ZK Circuits

| Circuit            | Proves                          | Private Input | Public Input                          |
|--------------------|---------------------------------|---------------|---------------------------------------|
| range_proof        | min <= amount <= max            | amount, salt  | min, max, amount_commitment           |
| stream_nullifier   | Knows secret → nullifier_hash  | secret        | stream_id, nullifier_hash             |

## Storage Strategy

| Tier        | Used for                                     |
|-------------|----------------------------------------------|
| instance()  | Admin, verifier address, stream count        |
| persistent()| Stream data, nullifiers, sender/recipient index |
| temporary() | (reserved for future batching)               |

## Security Properties

- Amount privacy: exact value never stored on-chain
- Anti double-spend: nullifier hash stored after first use
- Access control: require_auth() on all state mutations
- Proof soundness: Groth16 + BN254 pairing via Soroban Protocol 25
