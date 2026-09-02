# ZK Circuits Guide

## Prerequisites

```bash
# Install Circom
curl -fsSL https://github.com/iden3/circom/releases/latest/download/circom-linux-amd64 -o circom
chmod +x circom && sudo mv circom /usr/local/bin/

# Install snarkjs
npm install -g snarkjs

# Install circomlib (in circuits/ directory)
cd circuits && npm install circomlib
```

## Compile Circuits

```bash
# Range proof
circom circuits/range_proof/range_proof.circom --r1cs --wasm --sym -o circuits/range_proof/

# Nullifier
circom circuits/stream_nullifier/nullifier.circom --r1cs --wasm --sym -o circuits/stream_nullifier/
```

## Trusted Setup (dev only)

```bash
cd circuits/range_proof
snarkjs powersoftau new bn128 14 pot14_0000.ptau
snarkjs powersoftau contribute pot14_0000.ptau pot14_0001.ptau --name="dev"
snarkjs powersoftau prepare phase2 pot14_0001.ptau pot14_final.ptau
snarkjs groth16 setup range_proof.r1cs pot14_final.ptau range_proof_0000.zkey
snarkjs zkey contribute range_proof_0000.zkey range_proof_final.zkey --name="dev"
snarkjs zkey export verificationkey range_proof_final.zkey verification_key.json
```

## Generate a Proof (example)

```bash
# Create input.json
echo '{"amount": "1000", "salt": "12345", "min_amount": "1", "max_amount": "999999"}' > input.json

# Generate witness
node circuits/range_proof/range_proof_js/generate_witness.js \
  circuits/range_proof/range_proof_js/range_proof.wasm \
  input.json witness.wtns

# Generate proof
snarkjs groth16 prove range_proof_final.zkey witness.wtns proof.json public.json

# Verify proof (off-chain sanity check)
snarkjs groth16 verify verification_key.json public.json proof.json
```

## Convert for Soroban

Use the `circom2soroban` tool (see `tools/`) to convert `proof.json` +
`verification_key.json` into byte arrays compatible with the `zk_verifier` contract.
