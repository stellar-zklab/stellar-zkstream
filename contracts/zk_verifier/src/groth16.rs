//! Groth16 verification over BN254 using Soroban Protocol 25 crypto host functions
//! (soroban-sdk 25's `crypto::bn254` module — confirmed present and used for real below,
//! not just referenced in a comment).
//!
//! Implements the Groth16 pairing equation:
//!   e(A, B) = e(alpha, beta) * e(vk_x, gamma) * e(C, delta)
//!
//! where vk_x = IC[0] + sum(public_inputs[i] * IC[i+1])
//!
//! Rearranged into the multi-pairing-equals-one form the host function expects:
//!   e(A, B) * e(-alpha, beta) * e(-vk_x, gamma) * e(-C, delta) == 1
//!
//! Byte layout:
//!   Proof: A(64 bytes G1) | B(128 bytes G2) | C(64 bytes G1)
//!   VK:    alpha(64) | beta(128) | gamma(128) | delta(128) | IC[0](64) | IC[n](64 each)
//!
//! G1 points are 64-byte uncompressed (X||Y, 32 bytes each, big-endian). G2 points are
//! 128-byte uncompressed (X||Y, each an Fp2 element = 64 bytes, encoded c1||c0 per
//! soroban-sdk's Bn254G2Affine docs). This matches the standard Ethereum-style encoding
//! that snarkjs/circom Groth16 tooling produces.

use soroban_sdk::{
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr},
    Bytes, BytesN, Env, TryFromVal, Val, Vec,
};

pub const G1_SIZE: u32 = 64;
pub const G2_SIZE: u32 = 128;
/// Size of a BN254 scalar (Fr element), matching `public_inputs: &Vec<BytesN<32>>` below.
#[allow(dead_code)]
pub const SCALAR_SIZE: u32 = 32;

/// Parse and validate proof byte layout.
/// Returns (A_bytes, B_bytes, C_bytes) or panics on malformed input.
pub fn parse_proof(proof: &Bytes) -> (Bytes, Bytes, Bytes) {
    let expected = 2 * G1_SIZE + G2_SIZE;
    assert!(
        proof.len() >= expected,
        "proof too short: expected at least {} bytes",
        expected
    );
    let a = proof.slice(0..G1_SIZE);
    let b = proof.slice(G1_SIZE..G1_SIZE + G2_SIZE);
    let c = proof.slice(G1_SIZE + G2_SIZE..2 * G1_SIZE + G2_SIZE);
    (a, b, c)
}

/// Parse verification key and extract IC points.
pub fn parse_vk_ic(vk: &Bytes, num_inputs: u32) -> (Bytes, Bytes, Bytes, Bytes, Vec<Bytes>) {
    let alpha = vk.slice(0..G1_SIZE);
    let beta = vk.slice(G1_SIZE..G1_SIZE + G2_SIZE);
    let gamma = vk.slice(G1_SIZE + G2_SIZE..G1_SIZE + 2 * G2_SIZE);
    let delta = vk.slice(G1_SIZE + 2 * G2_SIZE..G1_SIZE + 3 * G2_SIZE);

    let ic_base = G1_SIZE + 3 * G2_SIZE;
    let mut ic_points: Vec<Bytes> = Vec::new(vk.env());
    for i in 0..=num_inputs {
        let offset = ic_base + i * G1_SIZE;
        ic_points.push_back(vk.slice(offset..offset + G1_SIZE));
    }
    (alpha, beta, gamma, delta, ic_points)
}

/// Converts a dynamically-sized `Bytes` slice (of the correct length) into a fixed-size
/// `BytesN<N>`. Panics if the slice isn't exactly N bytes — an appropriate failure mode
/// for a verifier: malformed-length curve point input should reject, not proceed.
fn to_fixed<const N: usize>(env: &Env, bytes: Bytes) -> BytesN<N> {
    let val: Val = bytes.into();
    BytesN::<N>::try_from_val(env, &val)
        .unwrap_or_else(|_| panic!("expected exactly {} bytes for curve point encoding", N))
}

fn g1(env: &Env, bytes: Bytes) -> Bn254G1Affine {
    Bn254G1Affine::from_bytes(to_fixed::<64>(env, bytes))
}

fn g2(env: &Env, bytes: Bytes) -> Bn254G2Affine {
    Bn254G2Affine::from_bytes(to_fixed::<128>(env, bytes))
}

/// Per soroban-sdk's Bn254G1Affine/G2Affine docs, all-zero bytes encode the point at
/// infinity (the group identity element). A legitimate Groth16 proof's A, B, C should
/// never legitimately be the identity — e(infinity, X) == 1 for any X, which trivially
/// satisfies the pairing check regardless of what the rest of the proof contains. Reject
/// it explicitly rather than relying on a real verification key never being degenerate.
fn is_zero(bytes: &Bytes) -> bool {
    let len = bytes.len();
    let mut i: u32 = 0;
    while i < len {
        if bytes.get(i).unwrap_or(0) != 0 {
            return false;
        }
        i += 1;
    }
    true
}

/// Verify a Groth16 BN254 proof for real.
pub fn verify(env: &Env, vk: &Bytes, proof: &Bytes, public_inputs: &Vec<BytesN<32>>) -> bool {
    // Validate proof structure
    if proof.len() < 2 * G1_SIZE + G2_SIZE {
        return false;
    }

    // Validate VK structure (must have IC[0] + one IC per public input)
    let expected_vk_len = G1_SIZE + 3 * G2_SIZE + (public_inputs.len() + 1) * G1_SIZE;
    if vk.len() < expected_vk_len {
        return false;
    }

    let (a_bytes, b_bytes, c_bytes) = parse_proof(proof);

    // Reject a proof whose A, B, or C is the point at infinity — see is_zero's doc
    // comment for why this can't be allowed to reach the pairing check.
    if is_zero(&a_bytes) || is_zero(&b_bytes) || is_zero(&c_bytes) {
        return false;
    }

    let (alpha_bytes, beta_bytes, gamma_bytes, delta_bytes, ic_bytes) =
        parse_vk_ic(vk, public_inputs.len());

    let a = g1(env, a_bytes);
    let b = g2(env, b_bytes);
    let c = g1(env, c_bytes);
    let alpha = g1(env, alpha_bytes);
    let beta = g2(env, beta_bytes);
    let gamma = g2(env, gamma_bytes);
    let delta = g2(env, delta_bytes);

    // Compute vk_x = IC[0] + sum(scalar_i * IC[i+1])
    let mut vk_x = g1(env, ic_bytes.get(0).unwrap());
    for i in 0..public_inputs.len() {
        let ic_point = g1(env, ic_bytes.get(i + 1).unwrap());
        let scalar = Fr::from_bytes(public_inputs.get(i).unwrap());
        let scaled = ic_point * scalar;
        vk_x = vk_x + scaled;
    }

    let neg_alpha = -alpha;
    let neg_vk_x = -vk_x;
    let neg_c = -c;

    let g1_points: Vec<Bn254G1Affine> = Vec::from_array(env, [a, neg_alpha, neg_vk_x, neg_c]);
    let g2_points: Vec<Bn254G2Affine> = Vec::from_array(env, [b, beta, gamma, delta]);

    env.crypto().bn254().pairing_check(g1_points, g2_points)
}
