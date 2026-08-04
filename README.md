# stellar-zkstream 🌊🔐

> **Privacy-Preserving Payment Streaming & Escrow Protocol on Stellar (Soroban)**  
> *Powered by Groth16 Zero-Knowledge Proofs & Soroban Protocol 25 BN254 Host Functions*

[![CI](https://github.com/stellar-zklab/stellar-zkstream/actions/workflows/ci.yml/badge.svg)](https://github.com/stellar-zklab/stellar-zkstream/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Stellar Drips Wave](https://img.shields.io/badge/Stellar-Drips%20Wave-blueviolet)](https://drips.network)
[![Soroban Version](https://img.shields.io/badge/Soroban-v22.0.0-orange)](https://developers.stellar.org)

---

## Executive Summary

`stellar-zkstream` is a privacy-preserving payment streaming protocol constructed natively on **Soroban**, Stellar's smart contract platform. It enables organizations, DAOs, and individuals to stream XLM or any SEP-41 compliant token to recipients continuously over time (for payroll, subscriptions, grants, and vesting) **without disclosing exact payment amounts or revealing recipient transaction links on-chain**.

By leveraging **Groth16 Zero-Knowledge Proofs (BN254 curve)** verified on-chain via **Soroban Protocol 25 native pairing host functions**, `stellar-zkstream` solves the core transparency trade-off of public blockchains: achieving verifiable, trustless financial execution while maintaining strict commercial and personal privacy.

---

## Key Features & Protocol Innovations

- 🔐 **Zero-Knowledge Amount Privacy**: Senders prove via ZK Range Proofs ($min \le \text{amount} \le max$) that payment streams are non-zero and fully funded without ever publishing the raw numerical value to the public ledger.
- 🚫 **Anti Double-Withdrawal Nullifiers**: Withdrawals use Poseidon cryptographic nullifier proofs ($N = \text{Poseidon}(s, \text{stream\_id})$). Once a nullifier is published and stored on-chain, it cannot be reused, preventing double-claiming.
- ⏱️ **Linear Continuous Vesting**: Tokens vest per-second based on ledger timestamps. Recipients can withdraw vested portions at any frequency without waiting for stream completion.
- 🔗 **Soroban Protocol 25 BN254 Native Host Integration**: Proof verification delegates cryptographic pairing calculations to Soroban's native `crypto().bn254_pairing_check()` host functions, reducing WASM execution footprint and gas costs by over 90%.
- ⚖️ **Revocable & Non-Revocable Streams**: Stream creators configure cancellation permissions at creation time. Cancelled streams automatically calculate vested funds owed to the recipient and return unvested remainders to the sender.
- 🧩 **SEP-41 Token Composability**: Native support for XLM and any standard Soroban token asset.

---

## Protocol Architecture & System Design

```
┌────────────────────────────────────────────────────────────────────────┐
│                              CLIENT SIDE                               │
│                                                                        │
│  Sender Input: (Amount, Duration, Salt)                                │
│       │                                                                │
│       ▼                                                                │
│  [range_proof.circom] ──► snarkjs (WASM) ──► Groth16 Proof (BN254)     │
│                                                   │                    │
└───────────────────────────────────────────────────┼────────────────────┘
                                                    │
                                   ┌────────────────┼────────────────────┐
                                   │    SOROBAN ON-CHAIN LAYER           │
                                   │                ▼                    │
                                   │  stream::create_stream()            │
                                   │        │                            │
                                   │        ▼                            │
                                   │  zk_verifier::vrfy_prf()            │
                                   │        │                            │
                                   │        │ Native Host Functions      │
                                   │        ▼                            │
                                   │  crypto().bn254_pairing_check()     │
                                   │        │                            │
                                   │        ▼ (Valid)                    │
                                   │  SEP-41 Token Escrow Transfer       │
                                   └─────────────────────────────────────┘
```

```
┌────────────────────────────────────────────────────────────────────────┐
│                           RECIPIENT CLAIM                              │
│                                                                        │
│  Recipient Secret (s) + Stream ID                                      │
│       │                                                                │
│       ▼                                                                │
│  [nullifier.circom] ──► Nullifier Proof ──► stream::withdraw()         │
│                                                   │                    │
│                                                   ▼                    │
│                                        Nullifier Registry Check        │
│                                        (Persistent Storage)            │
│                                                   │                    │
│                                                   ▼                    │
│                                        Vested Amount Transferred       │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Cryptographic & Mathematical Specification

### 1. Elliptic Curve Parameters (BN254 / alt_bn128)
The protocol operates over the BN254 pairing-friendly elliptic curve defined by $y^2 = x^3 + 3$ over prime field $\mathbb{F}_q$:
$$\text{Base Field } q = 21888242871839275222246405745257275088696311157297823662689037894645226208583$$
$$\text{Scalar Field } r = 21888242871839275222246405745257275088548364400416034343698204186575808495617$$

### 2. Groth16 Verification Equation
The `zk_verifier` contract evaluates the Groth16 zero-knowledge pairing equation:
$$e(A, B) = e(\alpha, \beta) \cdot e(vk_x, \gamma) \cdot e(C, \delta)$$

Where:
- $A \in \mathbb{G}_1$ (64 bytes), $B \in \mathbb{G}_2$ (128 bytes), $C \in \mathbb{G}_1$ (64 bytes) represent the proof.
- $\alpha, \beta, \gamma, \delta$ are the verification key parameters.
- $vk_x = \text{IC}_0 + \sum_{i=1}^{n} w_i \cdot \text{IC}_i$ is the public input linear combination.

### 3. Poseidon Hash Function
`stellar-zkstream` utilizes the ZK-friendly **Poseidon Hash** for commitments and nullifiers:
$$N = \text{Poseidon}(s, \text{stream\_id})$$
- State width $t = 3$, Full rounds $R_F = 8$, Partial rounds $R_P = 57$.

### 4. Linear Vesting Formula
At any ledger timestamp $t_{\text{now}}$, the claimable vested amount $V(t_{\text{now}})$ is computed as:
$$V(t_{\text{now}}) = \min\left( \text{TotalAmount}, \frac{\text{TotalAmount} \cdot (t_{\text{now}} - t_{\text{start}})}{t_{\text{end}} - t_{\text{start}}} \right) - \text{WithdrawnAmount}$$

---

## Smart Contract API Reference

### 1. `StreamContract` (`contracts/stream`)

#### `initialize(env: Env, admin: Address, verifier_contract: Address)`
Initializes contract state with administrator address and the target `zk_verifier` contract address.
- **Access Control**: `admin.require_auth()`
- **Storage Tier**: Writes to `instance()` storage.

#### `create_stream(env: Env, sender: Address, recipient: Address, token: Address, total_amount: i128, start_time: u64, end_time: u64, proof: Bytes, public_inputs: Vec<BytesN<32>>) -> u64`
Creates a payment stream, verifies the Groth16 ZK range proof, and escrows tokens.
- **Parameters**:
  - `sender`: Stream funder address.
  - `recipient`: Stream beneficiary address.
  - `token`: SEP-41 token contract address.
  - `total_amount`: Total token amount in stroops (i128).
  - `start_time` & `end_time`: Stream duration Unix timestamps.
  - `proof`: 256-byte Groth16 proof (`A | B | C`).
  - `public_inputs`: Public inputs vector.
- **Returns**: Unique `stream_id` (`u64`).

#### `withdraw(env: Env, stream_id: u64, caller: Address, nullifier_hash: BytesN<32>, nullifier_proof: Bytes, public_inputs: Vec<BytesN<32>>) -> i128`
Withdraws currently vested tokens for the recipient using a ZK nullifier proof.
- **Returns**: Amount withdrawn (`i128`).

#### `cancel_stream(env: Env, stream_id: u64, caller: Address)`
Cancels an active stream (sender only). Pays recipient vested portion and returns unvested remainder to sender.

#### `claimable_amount(env: Env, stream_id: u64) -> i128`
View function returning current withdrawable tokens for a stream ID without modifying state.

---

### 2. `ZkVerifierContract` (`contracts/zk_verifier`)

#### `vrfy_prf(env: Env, proof: Bytes, public_inputs: Vec<BytesN<32>>) -> bool`
Executes Groth16 BN254 verification using native host functions.
- **Proof format**: `A` (64 bytes) + `B` (128 bytes) + `C` (64 bytes) = 256 bytes total.

---

## Storage Architecture & Tier Strategy

| Storage Tier | Data Key | Content | Purpose |
|---|---|---|---|
| `instance()` | `DataKey::Admin` | `Address` | Admin governance address |
| `instance()` | `DataKey::VerifierContract` | `Address` | Deployed `zk_verifier` contract ID |
| `instance()` | `DataKey::StreamCount` | `u64` | Total stream counter |
| `persistent()`| `DataKey::Stream(id)` | `StreamData` | Stream state, balances, timestamps |
| `persistent()`| `DataKey::Nullifier(bytes32)` | `bool` | Anti double-withdrawal nullifier index |
| `persistent()`| `DataKey::StreamsBySender(address)` | `Vec<u64>` | Sender stream index map |
| `persistent()`| `DataKey::StreamsByRecipient(address)`| `Vec<u64>` | Recipient stream index map |

---

## Directory Structure

```
stellar-zkstream/
├── contracts/
│   ├── stream/             # Core payment streaming & escrow contract
│   ├── zk_verifier/        # Groth16 BN254 verifier contract
│   └── token_wrapper/      # SEP-41 token wrapper utilities
├── circuits/
│   ├── range_proof/        # Circom ZK range proof circuit (min <= amount <= max)
│   └── stream_nullifier/   # Circom ZK nullifier circuit (anti double-spend)
├── sdk/                    # TypeScript SDK
├── frontend/               # React application UI
├── tools/                  # CLI tooling (circom2soroban converter)
├── docs/                   # Architecture, ZK specs, deployment guides
└── scripts/
    └── deploy.sh           # Testnet deployment script
```

---

## Developer Quick Start

### Prerequisites

```bash
# 1. Rust + Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32v1-none

# 2. Stellar CLI
curl -fsSL https://github.com/stellar/stellar-cli/raw/main/install.sh | sh

# 3. Circom & snarkjs
npm install -g snarkjs
```

### Build & Test Contracts

```bash
# Clone the repository
git clone https://github.com/stellar-zklab/stellar-zkstream.git
cd stellar-zkstream

# Run unit tests
cargo test --all --features testutils

# Build release WASM binaries
cargo build --release --target wasm32v1-none
```

### Compile ZK Circuits

```bash
cd circuits

# Install circuit dependencies
npm install circomlib

# Compile range proof circuit
circom range_proof/range_proof.circom --r1cs --wasm --sym -o range_proof/

# Compile nullifier circuit
circom stream_nullifier/nullifier.circom --r1cs --wasm --sym -o stream_nullifier/
```

### Testnet Deployment

```bash
cp .env.example .env
# Configure STELLAR_ACCOUNT in .env
bash scripts/deploy.sh
```

---

## Security & Threat Model Analysis

| Threat Vector | Mitigation Strategy | Status |
|---|---|---|
| **Double Withdrawal Attack** | Nullifier hash checked against `persistent()` storage before withdrawal | ✅ Enforced on-chain |
| **Front-Running Claims** | Nullifiers bound to specific `stream_id` via Poseidon hash | ✅ Cryptographically Bound |
| **Proof Replay Attack** | Verification keys stored per contract instance & validated against inputs | ✅ Enforced |
| **Overflow / Precision Loss** | Fixed-point multiplication ordered to prevent overflow ($A \cdot t_{\text{elapsed}} / t_{\text{total}}$) | ✅ Verified |
| **Unauthorized Cancellation** | `require_auth()` enforced on `sender` address | ✅ Enforced |

---

## 🌊 Contributing — Stellar Drips Wave

`stellar-zkstream` participates in the **[Stellar Drips Wave](https://drips.network)** open-source reward program.

### Point System & Complexity Categories

| Category | Points | Tasks |
|---|---|---|
| 🔴 **High Complexity** | 200 pts | Groth16 host function verification, Circom circuits, nullifier logic |
| 🟡 **Medium Complexity** | 150 pts | SDK client development, React frontend pages, fuzzing tests |
| 🟢 **Trivial Complexity** | 100 pts | Documentation, deploy scripts, benchmarking tools |

### How to Contribute
1. Browse open issues tagged `stellardrips` on our [GitHub Issues Page](https://github.com/stellar-zklab/stellar-zkstream/issues).
2. Claim the issue on the [Drips Wave Dashboard](https://drips.network).
3. Implement, write unit tests, and submit a PR linking the issue number.

---

## License

Licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) for details.
