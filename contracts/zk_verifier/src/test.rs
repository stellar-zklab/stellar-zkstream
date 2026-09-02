#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

/// Build a mock VK of the correct byte length.
/// Layout: alpha(64) + beta(128) + gamma(128) + delta(128) + IC[0](64) + IC[1](64) = 576 bytes
fn mock_vk(env: &Env) -> Bytes {
    Bytes::from_array(env, &[0u8; 576])
}

/// Build a mock proof of the correct byte length.
/// Layout: A(64) + B(128) + C(64) = 256 bytes
fn mock_proof(env: &Env) -> Bytes {
    Bytes::from_array(env, &[0u8; 256])
}

#[test]
fn test_initialize_stores_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_get_vk_returns_stored_key() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    let vk = mock_vk(&env);
    client.initialize(&admin, &vk);
    assert_eq!(client.get_vk(), vk);
}

#[test]
fn test_update_vk_by_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    let new_vk = Bytes::from_array(&env, &[1u8; 576]);
    client.update_vk(&admin, &new_vk);
    assert_eq!(client.get_vk(), new_vk);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    client.initialize(&admin, &mock_vk(&env));
}

#[test]
fn test_verify_proof_rejects_all_zero_garbage() {
    // All-zero bytes decode to the point at infinity for both G1 and G2 (per
    // soroban-sdk's Bn254G1Affine/G2Affine docs). Pairing with infinity trivially
    // satisfies e(infinity, X) == 1 for any X, so without an explicit identity-element
    // check this would otherwise "verify" — see groth16::is_zero's doc comment. This
    // test exists specifically to pin that defensive check in place.
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));

    let proof = mock_proof(&env);
    let inputs: Vec<BytesN<32>> = Vec::new(&env);
    let result = client.vrfy_prf(&proof, &inputs);
    assert!(!result, "an all-zero garbage proof must not pass real verification");
}

mod real_proof {
    //! Generates a genuine Groth16 BN254 proof with arkworks for a trivial
    //! "prove knowledge of x such that x*x = y" circuit, serializes it into the exact
    //! byte layout groth16::verify() expects, and confirms the contract accepts a real
    //! valid proof and rejects a tampered one.
    use super::*;
    use ark_bn254::{Bn254, Fq, Fq2, Fr as ArkFr, G1Affine, G2Affine};
    use ark_ff::{BigInteger, PrimeField};
    use ark_groth16::Groth16;
    use ark_relations::lc;
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
    use ark_snark::SNARK;
    use ark_std::rand::{rngs::StdRng, SeedableRng};

    #[derive(Clone)]
    struct SquareCircuit {
        x: Option<ArkFr>,
        y: Option<ArkFr>,
    }

    impl ConstraintSynthesizer<ArkFr> for SquareCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<ArkFr>) -> Result<(), SynthesisError> {
            let x_var = cs.new_witness_variable(|| self.x.ok_or(SynthesisError::AssignmentMissing))?;
            let y_var = cs.new_input_variable(|| self.y.ok_or(SynthesisError::AssignmentMissing))?;
            cs.enforce_constraint(lc!() + x_var, lc!() + x_var, lc!() + y_var)?;
            Ok(())
        }
    }

    fn fq_to_be32(f: &Fq) -> [u8; 32] {
        let mut out = [0u8; 32];
        let bytes = f.into_bigint().to_bytes_be();
        out.copy_from_slice(&bytes);
        out
    }

    fn g1_to_bytes(env: &Env, p: &G1Affine) -> BytesN<64> {
        let mut out = [0u8; 64];
        out[0..32].copy_from_slice(&fq_to_be32(&p.x));
        out[32..64].copy_from_slice(&fq_to_be32(&p.y));
        BytesN::from_array(env, &out)
    }

    fn fq2_to_be64(f: &Fq2) -> [u8; 64] {
        // Soroban's Bn254G2Affine Fp2 encoding is be(c1) || be(c0) — imaginary part first.
        let mut out = [0u8; 64];
        out[0..32].copy_from_slice(&fq_to_be32(&f.c1));
        out[32..64].copy_from_slice(&fq_to_be32(&f.c0));
        out
    }

    fn g2_to_bytes(env: &Env, p: &G2Affine) -> BytesN<128> {
        let mut out = [0u8; 128];
        out[0..64].copy_from_slice(&fq2_to_be64(&p.x));
        out[64..128].copy_from_slice(&fq2_to_be64(&p.y));
        BytesN::from_array(env, &out)
    }

    fn fr_to_bytes(env: &Env, f: &ArkFr) -> BytesN<32> {
        let mut out = [0u8; 32];
        let bytes = f.into_bigint().to_bytes_be();
        out.copy_from_slice(&bytes);
        BytesN::from_array(env, &out)
    }

    /// Runs real Groth16 setup + prove for x=3, y=9, and returns
    /// (vk_bytes, proof_bytes, public_input_bytes) in the exact layout the contract expects.
    fn build_real_proof(env: &Env) -> (Bytes, Bytes, Vec<BytesN<32>>) {
        let mut rng = StdRng::seed_from_u64(42);

        let setup_circuit = SquareCircuit { x: None, y: None };
        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(setup_circuit, &mut rng)
            .expect("groth16 setup should succeed for this trivial circuit");

        let x = ArkFr::from(3u64);
        let y = ArkFr::from(9u64);
        let prove_circuit = SquareCircuit { x: Some(x), y: Some(y) };
        let proof = Groth16::<Bn254>::prove(&pk, prove_circuit, &mut rng)
            .expect("proving should succeed for a satisfied circuit");

        // Sanity-check against arkworks' own verifier before trusting our own serialization.
        assert!(
            Groth16::<Bn254>::verify(&vk, &[y], &proof).expect("arkworks verify should not error"),
            "arkworks' own verifier rejected the proof we just generated — setup/prove bug, not a serialization bug"
        );

        assert_eq!(vk.gamma_abc_g1.len(), 2, "expected IC[0] + one IC per public input");

        let mut vk_bytes = Bytes::new(env);
        vk_bytes.append(&Bytes::from(g1_to_bytes(env, &vk.alpha_g1)));
        vk_bytes.append(&Bytes::from(g2_to_bytes(env, &vk.beta_g2)));
        vk_bytes.append(&Bytes::from(g2_to_bytes(env, &vk.gamma_g2)));
        vk_bytes.append(&Bytes::from(g2_to_bytes(env, &vk.delta_g2)));
        vk_bytes.append(&Bytes::from(g1_to_bytes(env, &vk.gamma_abc_g1[0])));
        vk_bytes.append(&Bytes::from(g1_to_bytes(env, &vk.gamma_abc_g1[1])));

        let mut proof_bytes = Bytes::new(env);
        proof_bytes.append(&Bytes::from(g1_to_bytes(env, &proof.a)));
        proof_bytes.append(&Bytes::from(g2_to_bytes(env, &proof.b)));
        proof_bytes.append(&Bytes::from(g1_to_bytes(env, &proof.c)));

        let mut public_inputs: Vec<BytesN<32>> = Vec::new(env);
        public_inputs.push_back(fr_to_bytes(env, &y));

        (vk_bytes, proof_bytes, public_inputs)
    }

    #[test]
    fn test_verify_accepts_a_real_valid_groth16_proof() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let (vk_bytes, proof_bytes, public_inputs) = build_real_proof(&env);
        client.initialize(&admin, &vk_bytes);

        let result = client.vrfy_prf(&proof_bytes, &public_inputs);
        assert!(result, "a genuine, correctly-serialized Groth16 proof should verify");
    }

    #[test]
    fn test_verify_rejects_a_proof_for_the_wrong_public_input() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let (vk_bytes, proof_bytes, _correct_inputs) = build_real_proof(&env);
        client.initialize(&admin, &vk_bytes);

        // Claim y=10 instead of the real y=9 the proof was actually generated for.
        let wrong_y = ArkFr::from(10u64);
        let mut wrong_inputs: Vec<BytesN<32>> = Vec::new(&env);
        wrong_inputs.push_back(fr_to_bytes(&env, &wrong_y));

        let result = client.vrfy_prf(&proof_bytes, &wrong_inputs);
        assert!(!result, "a proof must not verify against a public input it wasn't generated for");
    }
}
