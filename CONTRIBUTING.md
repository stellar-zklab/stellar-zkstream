# Contributing to stellar-zkstream 🌊🔐

Welcome to the **`stellar-zkstream`** open-source repository! We are building the primary **privacy-preserving payment streaming and escrow protocol** on Soroban (Stellar).

Whether you are a Rust smart contract developer, a Zero-Knowledge circuit engineer (Circom/snarkjs), a TypeScript SDK builder, or a frontend contributor, your contributions help advance financial privacy for the entire Stellar ecosystem.

---

## 🚀 About the Protocol & Ecosystem Impact

`stellar-zkstream` enables organizations, DAOs, and individuals to stream XLM or any SEP-41 token continuously over time **without disclosing payment amounts or revealing recipient transaction links on-chain**.

By combining **Groth16 Zero-Knowledge Range Proofs** and **Soroban Protocol 25 BN254 Native Pairing Host Functions**, the protocol achieves high-throughput, low-cost confidential stream execution.

---

## 🗺️ Technical Architecture & Contribution Roadmap

Our development roadmap is structured across four milestone phases. We invite contributors to claim open issues corresponding to these phases:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     DEVELOPMENT ROADMAP PHASES                          │
│                                                                         │
│  Phase 1: Smart Contracts & ZK Circuits (Scaffolded & Verified)        │
│    ├── Core stream escrow & linear vesting math                        │
│    ├── Groth16 BN254 verifier contract                                 │
│    └── Circom range proof & nullifier circuits                          │
│                                                                         │
│  Phase 2: Client SDKs & Developer Tooling (Active Contribution)        │
│    ├── TypeScript SDK (@stellar-zklab/zkstream-sdk)                    │
│    ├── circom2soroban CLI proof byte converter                         │
│    └── Multi-wallet connectors (Freighter, Albedo, xBull)              │
│                                                                         │
│  Phase 3: Event Indexers & Frontend Applications (Upcoming)            │
│    ├── Soroban RPC event indexing daemon                               │
│    ├── React streaming dashboard & stream management UI                │
│    └── Payroll batch stream creation CLI                               │
│                                                                         │
│  Phase 4: Security Hardening & Mainnet Launch (Future)                 │
│    ├── Property-based fuzz testing with proptest                       │
│    ├── WASM bytecode size optimization (wasm-opt)                      │
│    └── Timelocked emergency multi-sig governance                       │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Developer Environment Quickstart

### Prerequisites
- **Rust Toolchain**: `rustup target add wasm32v1-none`
- **Stellar CLI**: v22.0.0+
- **Circom & snarkjs**: `npm install -g snarkjs`

### Build & Run Tests

```bash
# Clone the repository
git clone https://github.com/stellar-zklab/stellar-zkstream.git
cd stellar-zkstream

# Run unit tests across all contracts
cargo test --all --features testutils

# Compile release WASM binaries
cargo build --release --target wasm32v1-none
```

---

## 🌿 Git Branch & Conventional Commits

We follow Conventional Commits for transparent versioning:

| Prefix | Usage | Example |
|---|---|---|
| `feat:` | New feature or contract function | `feat(stream): add rate limiting per sender` |
| `fix:` | Bug fix or patch | `fix(verifier): resolve byte layout parsing error` |
| `docs:` | Documentation updates | `docs(readme): add Groth16 pairing equation` |
| `test:` | Unit test or integration test | `test(stream): add proptest fuzzing for vesting` |
| `ci:` | GitHub Actions CI changes | `ci: update Rust target matrix` |

---

## 📋 How to Claim an Issue & Submit a PR

1. **Pick an Issue**: Browse open tasks on our [GitHub Issues Page](https://github.com/stellar-zklab/stellar-zkstream/issues). Look for [`good-first-issue`](https://github.com/stellar-zklab/stellar-zkstream/issues?q=is%3Aissue+is%3Aopen+label%3A%22good-first-issue%22) if you are new to the codebase.
2. **Create a Branch**: `git checkout -b feat/your-feature-name`
3. **Verify Locally**: Ensure `cargo test --all --features testutils` passes cleanly.
4. **Submit PR**: Open a Pull Request referencing the issue number (e.g. `Closes #12`).

Thank you for building the future of private payments on Stellar! 🌊
