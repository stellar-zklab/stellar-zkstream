import { buildPoseidon } from "circomlibjs";
import { writeFileSync } from "fs";

const poseidon = await buildPoseidon();
const F = poseidon.F;

// range_proof: prove 0 < amount <= max, i.e. amount in [min_amount, max_amount].
// A real stream: amount=5_000_000 stroops, bounded by a sane range.
const amount = 5_000_000n;
const salt = 123456789n;
const min_amount = 1n;
const max_amount = 1_000_000_000n;
const commitmentField = poseidon([amount, salt]);
const amount_commitment = F.toObject(commitmentField).toString();

const rangeProofInput = {
  amount: amount.toString(),
  salt: salt.toString(),
  min_amount: min_amount.toString(),
  max_amount: max_amount.toString(),
  amount_commitment,
};
writeFileSync("build/range_proof/input.json", JSON.stringify(rangeProofInput, null, 2));

// nullifier: prove knowledge of `secret` behind nullifier_hash = Poseidon(secret, stream_id).
const secret = 987654321987n;
const stream_id = 0n;
const nullifierField = poseidon([secret, stream_id]);
const nullifier_hash = F.toObject(nullifierField).toString();

const nullifierInput = {
  secret: secret.toString(),
  stream_id: stream_id.toString(),
  nullifier_hash,
};
writeFileSync("build/nullifier/input.json", JSON.stringify(nullifierInput, null, 2));

console.log("range_proof input:", rangeProofInput);
console.log("nullifier input:", nullifierInput);
