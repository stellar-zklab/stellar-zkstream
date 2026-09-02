#!/usr/bin/env bash
# Deploys stellar-zkstream's contracts to Stellar testnet and records the resulting
# contract IDs in deployments/<network>.json. Requires the `stellar` CLI already
# installed and on PATH.
#
# zk_verifier is deployed but deliberately left un-initialized: `initialize()` needs a
# real Groth16 verification key, and the only one this repo can currently produce is the
# toy `x*x=y` demo circuit's VK generated in contracts/zk_verifier/src/test.rs — the real
# circuits in circuits/ (range_proof.circom, nullifier.circom) aren't compiled into a VK
# yet. Initializing with an empty or made-up key would just be a different kind of
# fabrication, so this step is left as an explicit manual follow-up — see
# docs/DEPLOYMENT_GUIDE.md.
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$REPO_ROOT/deployments"
OUT_FILE="$OUT_DIR/$NETWORK.json"
mkdir -p "$OUT_DIR"

echo "Deploying stellar-zkstream to Stellar $NETWORK..."

cd "$REPO_ROOT"
cargo build --release --target wasm32v1-none

if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM bytecode with wasm-opt -Oz..."
    wasm-opt -Oz target/wasm32v1-none/release/zk_verifier.wasm -o target/wasm32v1-none/release/zk_verifier.wasm || true
    wasm-opt -Oz target/wasm32v1-none/release/stream.wasm -o target/wasm32v1-none/release/stream.wasm || true
fi

if ! stellar keys ls | grep -q "^deployer$"; then
  echo "Generating deployer key..."
  stellar keys generate deployer --global
fi
stellar keys fund deployer --network "$NETWORK" || true
DEPLOYER_ADDR=$(stellar keys address deployer)

WASM_DIR="target/wasm32v1-none/release"

echo "Deploying zk_verifier (left un-initialized — see script header)..."
VERIFIER_ID=$(stellar contract deploy \
  --wasm "$WASM_DIR/zk_verifier.wasm" \
  --source deployer \
  --network "$NETWORK")

echo "Deploying stream escrow contract..."
STREAM_ID=$(stellar contract deploy \
  --wasm "$WASM_DIR/stream.wasm" \
  --source deployer \
  --network "$NETWORK")
stellar contract invoke --id "$STREAM_ID" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR" --verifier_contract "$VERIFIER_ID"

cat > "$OUT_FILE" <<EOF
{
  "network": "$NETWORK",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "$DEPLOYER_ADDR",
  "contracts": {
    "zk_verifier": "$VERIFIER_ID",
    "stream": "$STREAM_ID"
  },
  "notes": {
    "zk_verifier": "Deployed but NOT initialized — needs a real Groth16 verification key. See docs/DEPLOYMENT_GUIDE.md."
  }
}
EOF

echo ""
echo "Deployed to $NETWORK — recorded in $OUT_FILE"
cat "$OUT_FILE"
