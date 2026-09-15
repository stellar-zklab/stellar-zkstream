import { buildPoseidon } from "circomlibjs";
import { writeFileSync } from "fs";

const poseidon = await buildPoseidon();
const F = poseidon.F;

// The already-spent throwaway secret used for the original CLI verification test
// (tx 902139221671d507c26c434de592768d34606ebdb3edd90f358852ba3c19beae, per
// deployments/testnet.json) — its nullifier is already marked used on-chain, so
// publishing it as a worked example is safe. stream_id=1 matches demo_stream_1.
const secret = 999888777666555n;
const stream_id = 1n;
const nullifierField = poseidon([secret, stream_id]);
const nullifier_hash = F.toObject(nullifierField).toString();

const input = {
  secret: secret.toString(),
  stream_id: stream_id.toString(),
  nullifier_hash,
};
writeFileSync("build/nullifier/withdraw_input.json", JSON.stringify(input, null, 2));
console.log("safe withdraw_input.json:", input);
