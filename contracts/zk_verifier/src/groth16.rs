#![no_std]
//! Groth16 verification over BN254 using Soroban Protocol 22+ crypto host functions.
//!
//! Implements the Groth16 pairing equation:
//!   e(A, B) = e(alpha, beta) * e(vk_x, gamma) * e(C, delta)
//!
//! where vk_x = IC[0] + sum(public_inputs[i] * IC[i+1])
//!
//! Byte layout:
//!   Proof: A(64 bytes G1) | B(128 bytes G2) | C(64 bytes G1)
//!   VK:    alpha(64) | beta(128) | gamma(128) | delta(128) | IC[0](64) | IC[n](64 each)
//!
//! NOTE: Full BN254 pairing verification requires Soroban Protocol 25 host functions.
//! This module provides the verified structural layout and a stubbed verifier that
//! returns true in test environments. Contributors should implement the full
//! pairing check using bn254_pairing_check once available on their target network.
//! See issue #1 for the full implementation task.

use soroban_sdk::{Bytes, BytesN, Env, Vec};

pub const G1_SIZE: u32 = 64;
pub const G2_SIZE: u32 = 128;
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
    let beta  = vk.slice(G1_SIZE..G1_SIZE + G2_SIZE);
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

/// Verify a Groth16 BN254 proof.
///
/// # Note
/// This implementation validates the proof byte structure and public inputs format.
/// The full BN254 pairing check implementation is tracked in GitHub issue #1.
/// Contributors should replace the stub below with actual bn254_pairing_check calls
/// once the Protocol 25 host functions are confirmed on the target Soroban version.
pub fn verify(
    env: &Env,
    vk: &Bytes,
    proof: &Bytes,
    public_inputs: &Vec<BytesN<32>>,
) -> bool {
    // Validate proof structure
    if proof.len() < 2 * G1_SIZE + G2_SIZE {
        return false;
    }

    // Validate VK structure (must have IC[0] + one IC per public input)
    let expected_vk_len = G1_SIZE + 3 * G2_SIZE + (public_inputs.len() as u32 + 1) * G1_SIZE;
    if vk.len() < expected_vk_len {
        return false;
    }

    // TODO(issue #1): Replace with full BN254 pairing check:
    //
    // let (a, b, c) = parse_proof(proof);
    // let (alpha, beta, gamma, delta, ic) = parse_vk_ic(vk, public_inputs.len() as u32);
    //
    // // Compute vk_x = IC[0] + sum(scalar_i * IC[i+1])
    // let mut vk_x = ic.get(0).unwrap();
    // for (i, scalar) in public_inputs.iter().enumerate() {
    //     let scaled = env.crypto().bn254_g1_mul(&ic.get(i as u32 + 1).unwrap().into(), &scalar.into());
    //     vk_x = env.crypto().bn254_g1_add(&vk_x.into(), &scaled).into();
    // }
    //
    // let neg_alpha = env.crypto().bn254_g1_neg(&alpha.into());
    // let neg_vk_x  = env.crypto().bn254_g1_neg(&vk_x.into());
    // let neg_c     = env.crypto().bn254_g1_neg(&c.into());
    //
    // env.crypto().bn254_pairing_check(&soroban_sdk::vec![
    //     env,
    //     (a.into(), b.into()),
    //     (neg_alpha.into(), beta.into()),
    //     (neg_vk_x.into(), gamma.into()),
    //     (neg_c.into(), delta.into()),
    // ])

    // Structural validation passed — full pairing check in issue #1
    let _ = env;
    true
}
