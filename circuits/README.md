# Circuits

`range_proof/range_proof.circom` and `stream_nullifier/nullifier.circom` are the real
circuits `stream` needs: proving a streamed amount is in a valid range without revealing it,
and proving knowledge of a withdrawal secret without revealing it (double-withdrawal
prevention). Both are genuine Circom circuits using Poseidon (via `circomlib`) — not
placeholders.

## What's here

- `range_proof/range_proof.circom`, `stream_nullifier/nullifier.circom` — the circuit
  source.
- `build/range_proof/`, `build/nullifier/` — outputs of an actual Groth16 trusted-setup
  pipeline run against these circuits (see below). Committed: `*_vk.json`/`*_vk.hex` (the
  real verification keys — these are the ones `zk_verifier.initialize()` is actually loaded
  with on testnet), `*_final.zkey` (the real proving key, needed to generate a *new* proof
  without repeating the ceremony), `input.json`/`proof.json`/`public.json` (one real worked
  example: a genuine valid proof and the exact public inputs it verifies against). Not
  committed (regenerable, see `.gitignore`): the `.r1cs`/`.sym`/witness/`.wasm` compiler
  output and the Powers of Tau `.ptau` files.
- `gen_inputs.mjs` — computes a real Poseidon commitment/nullifier off-circuit (via
  `circomlibjs`) so the example witness actually satisfies the circuit's constraints,
  rather than using invented numbers that would fail witness generation.
- `convert_to_soroban.mjs` — converts a snarkjs VK/proof/public-signals triple into the
  exact byte layout `contracts/zk_verifier/src/groth16.rs` expects (G1 as 64-byte X‖Y,
  G2 as 128-byte with each Fp2 coordinate as c1‖c0 — snarkjs's own JSON uses `[c0, c1]`,
  so this deliberately swaps the pair).

## Reproducing the pipeline

```bash
npm install
circom range_proof/range_proof.circom -l . --r1cs --wasm --sym -o build/range_proof
circom stream_nullifier/nullifier.circom -l . --r1cs --wasm --sym -o build/nullifier

# Phase 1 (shared, universal — not circuit-specific)
cd build
npx snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
npx snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau -v -e="$(openssl rand -hex 32)"
npx snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

# Phase 2, per circuit
npx snarkjs groth16 setup range_proof/range_proof.r1cs pot12_final.ptau range_proof/range_proof_0000.zkey
npx snarkjs zkey contribute range_proof/range_proof_0000.zkey range_proof/range_proof_final.zkey -v -e="$(openssl rand -hex 32)"
npx snarkjs zkey export verificationkey range_proof/range_proof_final.zkey range_proof/range_proof_vk.json
# repeat groth16 setup / zkey contribute / export verificationkey for nullifier/

cd ..
node gen_inputs.mjs
node build/range_proof/range_proof_js/generate_witness.js build/range_proof/range_proof_js/range_proof.wasm build/range_proof/input.json build/range_proof/witness.wtns
node build/nullifier/nullifier_js/generate_witness.js build/nullifier/nullifier_js/nullifier.wasm build/nullifier/input.json build/nullifier/witness.wtns
npx snarkjs groth16 prove build/range_proof/range_proof_final.zkey build/range_proof/witness.wtns build/range_proof/proof.json build/range_proof/public.json
npx snarkjs groth16 prove build/nullifier/nullifier_final.zkey build/nullifier/witness.wtns build/nullifier/proof.json build/nullifier/public.json
npx snarkjs groth16 verify build/range_proof/range_proof_vk.json build/range_proof/public.json build/range_proof/proof.json
npx snarkjs groth16 verify build/nullifier/nullifier_vk.json build/nullifier/public.json build/nullifier/proof.json

node convert_to_soroban.mjs
```

`contracts/zk_verifier/src/test.rs`'s `real_zkstream_circuits` module hardcodes the
resulting VK/proof/public-input bytes and calls the actual contract's `vrfy_prf()` with
them — a passing test there is evidence this whole pipeline, including the byte
conversion, is correct end to end, not just internally self-consistent.

## Important caveat

This is a genuine, correctly-executed Groth16 setup — the math is real and the resulting
VK is real. It is **not** a production-grade multi-party ceremony: phase 1 and each
circuit's phase 2 had a single contributor (this pipeline run), not an independent
multi-party ceremony where no single party could have retained the toxic waste. Treat
these as real testnet verification keys, not something to point real funds at without a
proper multi-party ceremony first.
