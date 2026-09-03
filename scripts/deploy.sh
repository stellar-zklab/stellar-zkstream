#!/usr/bin/env bash
# Deploys stellar-zkstream's contracts to Stellar testnet and records the resulting
# contract IDs in deployments/<network>.json. Requires the `stellar` CLI already
# installed and on PATH.
#
# range_proof.circom and nullifier.circom are different circuits with different
# verification keys, so this deploys TWO zk_verifier instances — one contract can't
# correctly serve both. Each is initialized with the real VK produced by the trusted-setup
# pipeline documented in circuits/README.md (not a placeholder — see that doc for how to
# reproduce it, and contracts/zk_verifier/src/test.rs's real_zkstream_circuits module for
# proof this exact VK/proof pairing round-trips through the actual contract logic).
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$REPO_ROOT/deployments"
OUT_FILE="$OUT_DIR/$NETWORK.json"
mkdir -p "$OUT_DIR"

RANGE_PROOF_VK_FILE="$REPO_ROOT/circuits/build/range_proof/range_proof_vk.hex"
NULLIFIER_VK_FILE="$REPO_ROOT/circuits/build/nullifier/nullifier_vk.hex"
if [ ! -f "$RANGE_PROOF_VK_FILE" ] || [ ! -f "$NULLIFIER_VK_FILE" ]; then
  echo "Missing circuits/build/**/*_vk.hex — run the pipeline in circuits/README.md first." >&2
  exit 1
fi

echo "Deploying stellar-zkstream to Stellar $NETWORK..."

cd "$REPO_ROOT"
cargo build --release --target wasm32v1-none

if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM bytecode with wasm-opt -Oz..."
    wasm-opt -Oz target/wasm32v1-none/release/zk_verifier.wasm -o target/wasm32v1-none/release/zk_verifier.wasm || true
    wasm-opt -Oz target/wasm32v1-none/release/stream.wasm -o target/wasm32v1-none/release/stream.wasm || true
fi

if ! stellar keys address deployer >/dev/null 2>&1; then
  echo "Generating deployer key..."
  stellar keys generate deployer
fi
stellar keys fund deployer --network "$NETWORK" || true
DEPLOYER_ADDR=$(stellar keys address deployer)

WASM_DIR="target/wasm32v1-none/release"

deploy_verifier() {
  local vk_file="$1"
  local id
  id=$(stellar contract deploy \
    --wasm "$WASM_DIR/zk_verifier.wasm" \
    --source deployer \
    --network "$NETWORK")
  stellar contract invoke --id "$id" --source deployer --network "$NETWORK" \
    -- initialize --admin "$DEPLOYER_ADDR" --verification_key "$(cat "$vk_file")" >&2
  echo "$id"
}

echo "Deploying zk_verifier for range_proof, initialized with its real VK..."
RANGE_VERIFIER_ID=$(deploy_verifier "$RANGE_PROOF_VK_FILE")

echo "Deploying zk_verifier for nullifier, initialized with its real VK..."
NULLIFIER_VERIFIER_ID=$(deploy_verifier "$NULLIFIER_VK_FILE")

echo "Deploying stream escrow contract..."
STREAM_ID=$(stellar contract deploy \
  --wasm "$WASM_DIR/stream.wasm" \
  --source deployer \
  --network "$NETWORK")
stellar contract invoke --id "$STREAM_ID" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR" --range_verifier "$RANGE_VERIFIER_ID" --nullifier_verifier "$NULLIFIER_VERIFIER_ID"

cat > "$OUT_FILE" <<EOF
{
  "network": "$NETWORK",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "$DEPLOYER_ADDR",
  "contracts": {
    "range_proof_verifier": "$RANGE_VERIFIER_ID",
    "nullifier_verifier": "$NULLIFIER_VERIFIER_ID",
    "stream": "$STREAM_ID"
  },
  "notes": {
    "range_proof_verifier": "zk_verifier initialized with the real range_proof.circom VK.",
    "nullifier_verifier": "zk_verifier initialized with the real nullifier.circom VK."
  }
}
EOF

echo ""
echo "Deployed to $NETWORK — recorded in $OUT_FILE"
cat "$OUT_FILE"
