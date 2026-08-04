#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../.env"
NETWORK="${STELLAR_NETWORK:-testnet}"
echo "Deploying to $NETWORK..."

cargo build --release --target wasm32v1-none

VERIFIER_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/zk_verifier.wasm \
  --source "$STELLAR_ACCOUNT" --network "$NETWORK")
echo "zk_verifier: $VERIFIER_ID"

STREAM_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/stream.wasm \
  --source "$STELLAR_ACCOUNT" --network "$NETWORK")
echo "stream: $STREAM_ID"

stellar contract invoke --id "$VERIFIER_ID" \
  --source "$STELLAR_ACCOUNT" --network "$NETWORK" \
  -- initialize \
  --admin "$(stellar keys address "$STELLAR_ACCOUNT")" \
  --verification_key "0000"

stellar contract invoke --id "$STREAM_ID" \
  --source "$STELLAR_ACCOUNT" --network "$NETWORK" \
  -- initialize \
  --admin "$(stellar keys address "$STELLAR_ACCOUNT")" \
  --verifier_contract "$VERIFIER_ID"

echo "Done! Update your .env with:"
echo "  VERIFIER_CONTRACT_ID=$VERIFIER_ID"
echo "  STREAM_CONTRACT_ID=$STREAM_ID"
