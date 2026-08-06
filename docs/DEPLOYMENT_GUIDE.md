# stellar-zkstream Soroban Protocol 25 Deployment Guide 🌊🔐

This guide details how to deploy `stellar-zkstream` smart contracts to **Stellar Testnet** using Soroban Protocol 25 native BN254 host functions.

## 📋 Prerequisites
- **Stellar CLI**: v22.0.0+ (`cargo install --locked stellar-cli`)
- **Rust Wasm Target**: `rustup target add wasm32v1-none`
- **Binary Optimizer**: `wasm-opt` (from binaryen toolchain)

## 🌐 Network Configuration
- **Network**: `testnet`
- **RPC URL**: `https://soroban-testnet.stellar.org:443`
- **Network Passphrase**: `"Test SDF Network ; September 2015"`

## 🚀 Step-by-Step Deployment

### 1. Generate & Fund Deployer Key
```bash
stellar keys generate deployer
stellar keys fund deployer --network testnet
```

### 2. Run Automated Deployment Script
```bash
bash scripts/deploy.sh
```

### 3. Initialize Stream Contract
```bash
stellar contract invoke \
  --id <STREAM_CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- \
  initialize \
  --admin <DEPLOYER_ADDRESS> \
  --verifier_contract <VERIFIER_CONTRACT_ID>
```
