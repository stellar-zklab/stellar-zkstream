// Converts a snarkjs Groth16 verification key / proof / public-signals triple into the
// exact byte layout contracts/zk_verifier/src/groth16.rs expects:
//   G1 = X(32) || Y(32) big-endian, 64 bytes.
//   G2 = X(64) || Y(64), where each Fp2 coordinate is c1(32) || c0(32) — imaginary part
//        first, per soroban-sdk's Bn254G2Affine docs. snarkjs's own JSON encodes each Fp2
//        coordinate as [c0, c1] (real first), so this deliberately swaps the pair.
//   VK  = alpha(64) | beta(128) | gamma(128) | delta(128) | IC[0](64) | IC[1..n](64 each)
//   Proof = A(64) | B(128) | C(64)
import { readFileSync, writeFileSync } from "fs";

function feToBytes32(decStr) {
  const n = BigInt(decStr);
  return n.toString(16).padStart(64, "0");
}

function g1ToHex([x, y, _z]) {
  return feToBytes32(x) + feToBytes32(y);
}

function g2ToHex([[xc0, xc1], [yc0, yc1], _z]) {
  // Contract wants c1||c0 (imaginary first); snarkjs gives [c0, c1].
  return feToBytes32(xc1) + feToBytes32(xc0) + feToBytes32(yc1) + feToBytes32(yc0);
}

function convert(dir, circuitName) {
  const vk = JSON.parse(readFileSync(`${dir}/${circuitName}_vk.json`, "utf8"));
  const proof = JSON.parse(readFileSync(`${dir}/proof.json`, "utf8"));
  const publicSignals = JSON.parse(readFileSync(`${dir}/public.json`, "utf8"));

  const alpha = g1ToHex(vk.vk_alpha_1);
  const beta = g2ToHex(vk.vk_beta_2);
  const gamma = g2ToHex(vk.vk_gamma_2);
  const delta = g2ToHex(vk.vk_delta_2);
  const ic = vk.IC.map(g1ToHex).join("");
  const vkHex = alpha + beta + gamma + delta + ic;

  const a = g1ToHex(proof.pi_a);
  const b = g2ToHex(proof.pi_b);
  const c = g1ToHex(proof.pi_c);
  const proofHex = a + b + c;

  const publicInputsHex = publicSignals.map(feToBytes32);

  writeFileSync(`${dir}/${circuitName}_vk.hex`, vkHex);
  writeFileSync(`${dir}/${circuitName}_proof.hex`, proofHex);
  writeFileSync(`${dir}/${circuitName}_public_inputs.json`, JSON.stringify(publicInputsHex, null, 2));

  console.log(`${circuitName}: vk ${vkHex.length / 2} bytes, proof ${proofHex.length / 2} bytes, ${publicInputsHex.length} public inputs`);
  console.log(`  expected vk len = 64 + 3*128 + ${vk.IC.length}*64 = ${64 + 3 * 128 + vk.IC.length * 64}`);
}

convert("build/range_proof", "range_proof");
convert("build/nullifier", "nullifier");
