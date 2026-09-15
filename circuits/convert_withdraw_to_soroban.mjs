// Same conversion logic as convert_to_soroban.mjs, applied to the withdraw_* example
// files instead of the generic circuit demo files.
import { readFileSync, writeFileSync } from "fs";

function feToBytes32(decStr) {
  const n = BigInt(decStr);
  return n.toString(16).padStart(64, "0");
}

function g1ToHex([x, y, _z]) {
  return feToBytes32(x) + feToBytes32(y);
}

function g2ToHex([[xc0, xc1], [yc0, yc1], _z]) {
  return feToBytes32(xc1) + feToBytes32(xc0) + feToBytes32(yc1) + feToBytes32(yc0);
}

const dir = "build/nullifier";
const proof = JSON.parse(readFileSync(`${dir}/withdraw_proof.json`, "utf8"));
const publicSignals = JSON.parse(readFileSync(`${dir}/withdraw_public.json`, "utf8"));

const a = g1ToHex(proof.pi_a);
const b = g2ToHex(proof.pi_b);
const c = g1ToHex(proof.pi_c);
const proofHex = a + b + c;

const publicInputsHex = publicSignals.map(feToBytes32);

writeFileSync(`${dir}/withdraw_proof.hex`, proofHex);
writeFileSync(`${dir}/withdraw_public_inputs.json`, JSON.stringify(publicInputsHex, null, 2));

console.log(`withdraw proof: ${proofHex.length / 2} bytes, ${publicInputsHex.length} public inputs`);
console.log("public inputs:", publicInputsHex);
