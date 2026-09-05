#!/usr/bin/env bash
# Targeted redeploy of ONLY the stream contract, after fixing create_batch_streams'
# dropped proof verification and adding a re-init guard. Both zk_verifier deployments
# (range_proof_verifier, nullifier_verifier) are unchanged and keep their existing
# testnet addresses — their code did not change, so this script does not redeploy them,
# it just points the new stream contract at the SAME two verifier addresses.
#
# Any streams that existed on the pre-fix stream contract are stranded there (testnet
# only, no real value) — their sender/recipient can still interact with the OLD contract
# ID directly if needed, but the new deployment starts with zero streams.
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOYMENTS_FILE="$REPO_ROOT/deployments/$NETWORK.json"

if [ ! -f "$DEPLOYMENTS_FILE" ]; then
  echo "Expected an existing $DEPLOYMENTS_FILE to merge into — run scripts/deploy.sh first if this is a fresh environment." >&2
  exit 1
fi
if ! command -v jq &> /dev/null; then
  echo "This script needs 'jq' to safely merge the new address into $DEPLOYMENTS_FILE (apt install jq / brew install jq)." >&2
  exit 1
fi

echo "Redeploying stream to Stellar $NETWORK (range_proof_verifier and nullifier_verifier untouched)..."

if ! stellar keys address deployer >/dev/null 2>&1; then
  echo "No 'deployer' identity found. This script expects the SAME deployer used for the original deployment." >&2
  exit 1
fi
stellar keys fund deployer --network "$NETWORK" || true
DEPLOYER_ADDR=$(stellar keys address deployer)
EXPECTED_DEPLOYER=$(jq -r '.deployer' "$DEPLOYMENTS_FILE")
if [ "$DEPLOYER_ADDR" != "$EXPECTED_DEPLOYER" ]; then
  echo "WARNING: local 'deployer' identity ($DEPLOYER_ADDR) does not match the deployer recorded in $DEPLOYMENTS_FILE ($EXPECTED_DEPLOYER)." >&2
fi

RANGE_VERIFIER_ID=$(jq -r '.contracts.range_proof_verifier' "$DEPLOYMENTS_FILE")
NULLIFIER_VERIFIER_ID=$(jq -r '.contracts.nullifier_verifier' "$DEPLOYMENTS_FILE")
OLD_STREAM_ID=$(jq -r '.contracts.stream' "$DEPLOYMENTS_FILE")

cd "$REPO_ROOT"
cargo build --release --target wasm32v1-none --package stream
if command -v wasm-opt &> /dev/null; then
  wasm-opt -Oz target/wasm32v1-none/release/stream.wasm -o target/wasm32v1-none/release/stream.wasm || true
fi

NEW_STREAM_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/stream.wasm \
  --source deployer \
  --network "$NETWORK")
stellar contract invoke --id "$NEW_STREAM_ID" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR" --range_verifier "$RANGE_VERIFIER_ID" --nullifier_verifier "$NULLIFIER_VERIFIER_ID"

TMP_FILE="$(mktemp)"
jq \
  --arg new_id "$NEW_STREAM_ID" \
  --arg old_id "$OLD_STREAM_ID" \
  --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '.contracts.stream = $new_id
   | .notes.stream = ("Redeployed " + $ts + " after fixing create_batch_streams (previously discarded its proof/public_inputs entirely, bypassing ZK-gating for batch-created streams — now verifies one proof per stream) and adding a re-init guard — " + $old_id + " was the pre-fix instance and is stale. Points at the SAME range_proof_verifier/nullifier_verifier as before; those contracts did not change. Any streams that existed on " + $old_id + " are not migrated.")' \
  "$DEPLOYMENTS_FILE" > "$TMP_FILE"
mv "$TMP_FILE" "$DEPLOYMENTS_FILE"

for f in README.md sdk/README.md frontend/src/soroban.ts; do
  if [ -f "$REPO_ROOT/$f" ] && grep -q "$OLD_STREAM_ID" "$REPO_ROOT/$f"; then
    sed -i "s/$OLD_STREAM_ID/$NEW_STREAM_ID/g" "$REPO_ROOT/$f"
    echo "Patched $f"
  fi
done

echo ""
echo "Redeployed stream: $OLD_STREAM_ID -> $NEW_STREAM_ID"
echo "Updated: $DEPLOYMENTS_FILE"
echo ""
echo "Next steps (not done by this script):"
echo "  - review: git -C \"$REPO_ROOT\" diff"
echo "  - git add -A && git commit"
