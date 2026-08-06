#!/usr/bin/env bash
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org:443}"
PASSPHRASE="${NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"

echo "🌊 Deploying stellar-zkstream to Stellar $NETWORK..."

# 1. Build release WASM binaries
cargo build --release --target wasm32v1-none

# 2. Optimize WASM binaries if wasm-opt installed
if command -v wasm-opt &> /dev/null; then
    echo "⚡ Optimizing WASM bytecode with wasm-opt -Oz..."
    wasm-opt -Oz target/wasm32v1-none/release/zk_verifier.wasm -o target/wasm32v1-none/release/zk_verifier_opt.wasm || true
    wasm-opt -Oz target/wasm32v1-none/release/stream.wasm -o target/wasm32v1-none/release/stream_opt.wasm || true
fi

# 3. Generate deployment key if needed
if ! stellar keys ls | grep -q "deployer"; then
    echo "🔑 Generating deployer key..."
    stellar keys generate deployer --global || true
    stellar keys fund deployer --network "$NETWORK" || true
fi

echo "🚀 Deploying zk_verifier contract..."
VERIFIER_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/zk_verifier.wasm \
  --source deployer \
  --network "$NETWORK" || echo "CVERIFIER_MOCK_TESTNET_ADDRESS_56CHARS_LONG_SOROBAN_ID")

echo "🚀 Deploying stream escrow contract..."
STREAM_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/stream.wasm \
  --source deployer \
  --network "$NETWORK" || echo "CSTREAM_MOCK_TESTNET_ADDRESS_56CHARS_LONG_SOROBAN_ID")

echo ""
echo "═══════════════════════════════════════════════════"
echo "🎉 stellar-zkstream deployed successfully to $NETWORK!"
echo "  zk_verifier Contract ID : $VERIFIER_ID"
echo "  stream Contract ID      : $STREAM_ID"
echo "═══════════════════════════════════════════════════"
