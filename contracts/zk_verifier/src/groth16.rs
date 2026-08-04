#![no_std]
//! Groth16 verification over BN254 using Soroban Protocol 25 native host functions.
//!
//! Verifies: e(A,B) = e(alpha,beta) * e(vk_x,gamma) * e(C,delta)
//! where vk_x = IC[0] + sum(public_inputs[i] * IC[i+1])
//!
//! Proof layout  : A(64b G1) | B(128b G2) | C(64b G1)   = 256 bytes
//! VK layout     : alpha(64) | beta(128) | gamma(128) | delta(128) | IC[0](64) | IC[i](64 each)

use soroban_sdk::{Bytes, BytesN, Env, Vec};

const G1: u32 = 64;
const G2: u32 = 128;

pub fn verify(
    env: &Env,
    vk: &Bytes,
    proof: &Bytes,
    public_inputs: &Vec<BytesN<32>>,
) -> bool {
    // Parse proof
    let a = proof.slice(0..G1);
    let b = proof.slice(G1..G1 + G2);
    let c = proof.slice(G1 + G2..2 * G1 + G2);

    // Parse VK
    let alpha = vk.slice(0..G1);
    let beta  = vk.slice(G1..G1 + G2);
    let gamma = vk.slice(G1 + G2..G1 + 2 * G2);
    let delta = vk.slice(G1 + 2 * G2..G1 + 3 * G2);

    // Compute vk_x = IC[0] + sum(scalar_i * IC[i+1])
    let ic_base = G1 + 3 * G2;
    let mut vk_x = vk.slice(ic_base..ic_base + G1);

    for (i, scalar) in public_inputs.iter().enumerate() {
        let off = ic_base + G1 + (i as u32 + 1) * G1;
        let ic_i = vk.slice(off..off + G1);
        let scaled = env.crypto().bn254_g1_mul(&ic_i.into(), &scalar.into());
        vk_x = env.crypto().bn254_g1_add(&vk_x.into(), &scaled).into();
    }

    // Pairing check: e(A,B) * e(-alpha,beta) * e(-vk_x,gamma) * e(-C,delta) == 1
    let neg_alpha = env.crypto().bn254_g1_neg(&alpha.into());
    let neg_vk_x  = env.crypto().bn254_g1_neg(&vk_x.into());
    let neg_c     = env.crypto().bn254_g1_neg(&c.into());

    env.crypto().bn254_pairing_check(&soroban_sdk::vec![
        env,
        (a.into(), b.into()),
        (neg_alpha.into(), beta.into()),
        (neg_vk_x.into(), gamma.into()),
        (neg_c.into(), delta.into()),
    ])
}
