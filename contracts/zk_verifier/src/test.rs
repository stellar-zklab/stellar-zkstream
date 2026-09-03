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

/// Verifies this contract's `vrfy_prf()` against VKs and proofs generated by an actual,
/// complete Groth16 trusted-setup pipeline (circom -> snarkjs powers-of-tau -> phase 2 ->
/// contribution -> export) for the project's real `range_proof.circom` and `nullifier.circom`
/// circuits — not the toy `SquareCircuit` above, and not hand-crafted bytes. Each proof/VK
/// pair was independently confirmed valid by `snarkjs groth16 verify` before being converted
/// to this contract's byte layout, so a pass here is real evidence the byte conversion
/// (endianness, G2 c1||c0 ordering, IC layout) is actually correct end to end, not just
/// internally self-consistent. See circuits/convert_to_soroban.mjs for how these bytes were
/// produced from circuits/build/{range_proof,nullifier}/*_vk.json and proof.json.
mod real_zkstream_circuits {
    use super::*;
    use soroban_sdk::{TryFromVal, Val};

    fn hex_to_bytes(env: &Env, hex: &str) -> Bytes {
        fn nibble(c: u8) -> u8 {
            match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => panic!("invalid hex digit"),
            }
        }
        let hex_bytes = hex.as_bytes();
        let mut out = Bytes::new(env);
        let mut i = 0;
        while i + 1 < hex_bytes.len() {
            let byte = (nibble(hex_bytes[i]) << 4) | nibble(hex_bytes[i + 1]);
            out.push_back(byte);
            i += 2;
        }
        out
    }

    fn hex_to_scalar(env: &Env, hex: &str) -> BytesN<32> {
        let bytes: Val = hex_to_bytes(env, hex).into();
        BytesN::<32>::try_from_val(env, &bytes).unwrap()
    }

    // circuits/build/range_proof/range_proof_vk.hex / _proof.hex / _public_inputs.json,
    // for amount=5_000_000 in range [1, 1_000_000_000] with a real Poseidon commitment.
    const RANGE_PROOF_VK_HEX: &str = "19bbb5cd9a21c17cbe82877a236c2ddd0607f75413acda36dde6674817daea4a1c1669920bd3d11d1b8ae419b31c6311f992902f101bd6ef6895418af964acb50b3320f1892e3d4fbdb7c0d1998f0a25e7858874b290869297ed32ef7fb8b8512e19d67231cc9b5536a6e51d91b7a0a4d4d00d4efaad1bf3d1d7601d264ff6f5211d4047a36fe9ac11493215db88b75a523531f2ff705fe9431c851415596c5b02cc6e7c22d1fd67a89cd1eaf7190f4d4bb9e5c87af223ef403346d170bd2669198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa075941922ea4b32bb21c25eca103353657c58d2b7c98d19bda5f74065dfdb4da080c4476437b02ca6d236c0901aae6f4cb8c492efcbc274b1cf7d0e2a784413d042b3f9360659602711be8c528473728999622ed54bf313bb600d31cf509ddb91a8b06bb8997576362769e6afa57a34ff4eafd612187ff5863be975dfa1e66da2e1c3884bc17899615b632d7273d9e88383205973ab6211ec54a2defc53c4784156d4a36f483868a6989fb1a8b23f0fcc3d1b95cf50ed0451605a6e93a1ec8ed1aa2c770b46bb5c7942ae637dcaf72eb331fef74850ecc55d1dee5626a02ae750c254f0eae4acfaa826ec8de43786f7ef4ed9b9b981131b7c7a71a352e19ee202ebb510c6053ef5825484047b337f7537f59693bf7ac0f37eb8d90e3ec5d2b5228a33e82eeb3fcdcc24ccb852a73dc10f1ddf3bdff0848523f1b3da9fe2343521877ab33cafe7d841b6746d0ce0389e3ee3e36b7ef53ee530d0e24055b963363126c070a7b8f7a6bea4b5cc96c51f8365fcee15bbbead5cfbf1d433a35be79c32f5d1cea31050533fb3014dce064568bfa78c13cfd2b77e0aa676206df99bdfe216930d4786ef7c6839aed879d25b9af0fb9314445c4d5256ef11e8b468cbbc0";
    const RANGE_PROOF_PROOF_HEX: &str = "1576394dd31e3810dc98e1fd21ab18aedded58d5a5918eb8719e7285cc6f5c4613a756208462ceddbf4b4dcf46f3e8544b10f1838eb2de2ece5dba021d61a82b2543a7eb2ad496caa3ea17e29f0ff89ba9db63f69c12652d389a6ab584ee6d950e362741df18ca9ba3727de772be461afe17f79de97e562214892a5aeb3a4f592eded200fc7f792efbff1db16b68bffa910df19d68c244fb19812e109d43fb6516b288ee6ccde0aa211f20eb79553038b1c207e24517b791e125389beeb85040044a48c43d7df994c4d5faf4eb943e96dcc3247bfe6d0ee4437e0b11b814121d1d871314e79e7fbe0f9e1170749f441a8c68bfd7caf8ebfeb7ac76af49ba612b";
    const RANGE_PROOF_PUBLIC_INPUTS_HEX: [&str; 4] = [
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000001",
        "000000000000000000000000000000000000000000000000000000003b9aca00",
        "2a50431f0c3168c0be1bbb5405b319c02a868a696b208557af125d46fb3ab4f0",
    ];

    // circuits/build/nullifier/nullifier_vk.hex / _proof.hex / _public_inputs.json,
    // for nullifier_hash = Poseidon(secret, stream_id=0).
    const NULLIFIER_VK_HEX: &str = "19bbb5cd9a21c17cbe82877a236c2ddd0607f75413acda36dde6674817daea4a1c1669920bd3d11d1b8ae419b31c6311f992902f101bd6ef6895418af964acb50b3320f1892e3d4fbdb7c0d1998f0a25e7858874b290869297ed32ef7fb8b8512e19d67231cc9b5536a6e51d91b7a0a4d4d00d4efaad1bf3d1d7601d264ff6f5211d4047a36fe9ac11493215db88b75a523531f2ff705fe9431c851415596c5b02cc6e7c22d1fd67a89cd1eaf7190f4d4bb9e5c87af223ef403346d170bd2669198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa116c23d3fad89e967c658279fac28c57ac9d393c580484e722706cd056721ed11d29243d438b8b3bf222ce00fccb78593a901f412c480cd0137cf53671ccba292f8d02b241707b7a237206345e03ab24f2f59bc2b62aeeebfdd075c3780076762373420efae27afbd201169f78e3d5fefb9c2dec97003d41ccc34a6a49af87d827c8acc86f422dde3115f834c0c31fd994167a2eda084e4f874479239a0d8f402ad72e9a78acc2803e35d58402426bea57e2ea7fa0be35b8ebca9487241ec5e11e062df10fa03e03b3f4642b8bd7cf7cc20250ef4369f173f9e2c12337716731255e037c1fba6666fc480a00fe6286df3b500b8ea3468b00902f2fd9e4f683591e00a66562437d1b71a7471bdba2a233020c67d2b79364eafc9a1a156c0feb1a1af1d69ece6420190698513fcba968b18834a2623c49fd2829457785ece7193a";
    const NULLIFIER_PROOF_HEX: &str = "07b8e9fcbc3bd6b540816cbb34c881abd9eaf059d58d3f3d2cf1f0f797da3a6b11ee0d228411d8103bdca43184bbe65d7ee7c8a7ebccbded20def4a01253e40c06cbaa9ae8a1540083e005745ca38691c79e83bf1cb3ed2d09fc059d955b24c00c8373e4eaf89ac7cd6fd592a64a0c5faea8d2a3baf9aa854a6198f80eee78a8083edf4286f1ffa19c1b6848a25761f656a2e9ab600330962417adc819812c2c247ab9de355b54241bcd4cb9ee43680a90df496e7c7bddbecf5f6bd4448b1e822297d242cf0ecd37bd03d768b99e0fa3c5446bd1cc9c007e740760b529dad794015c94859d38570dd8b32857de3ec4b95d57f3af7872f9fe7df13fe0cb5b2158";
    const NULLIFIER_PUBLIC_INPUTS_HEX: [&str; 2] = [
        "0000000000000000000000000000000000000000000000000000000000000000",
        "01932d5bf06dc9929e84d1d97e006822f4c38b99f8e406929d06f5043337d286",
    ];

    #[test]
    fn test_verify_accepts_the_real_range_proof_circuit() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let vk = hex_to_bytes(&env, RANGE_PROOF_VK_HEX);
        client.initialize(&admin, &vk);

        let proof = hex_to_bytes(&env, RANGE_PROOF_PROOF_HEX);
        let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
        for h in RANGE_PROOF_PUBLIC_INPUTS_HEX {
            inputs.push_back(hex_to_scalar(&env, h));
        }

        let result = client.vrfy_prf(&proof, &inputs);
        assert!(result, "a real proof for the real range_proof circuit, against its own real VK, must verify");
    }

    #[test]
    fn test_verify_accepts_the_real_nullifier_circuit() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let vk = hex_to_bytes(&env, NULLIFIER_VK_HEX);
        client.initialize(&admin, &vk);

        let proof = hex_to_bytes(&env, NULLIFIER_PROOF_HEX);
        let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
        for h in NULLIFIER_PUBLIC_INPUTS_HEX {
            inputs.push_back(hex_to_scalar(&env, h));
        }

        let result = client.vrfy_prf(&proof, &inputs);
        assert!(result, "a real proof for the real nullifier circuit, against its own real VK, must verify");
    }

    #[test]
    fn test_verify_rejects_the_real_range_proof_against_a_tampered_public_input() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let vk = hex_to_bytes(&env, RANGE_PROOF_VK_HEX);
        client.initialize(&admin, &vk);

        let proof = hex_to_bytes(&env, RANGE_PROOF_PROOF_HEX);
        let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
        for h in RANGE_PROOF_PUBLIC_INPUTS_HEX {
            inputs.push_back(hex_to_scalar(&env, h));
        }
        // Claim a different max_amount than the one this proof was actually generated for.
        inputs.set(2, hex_to_scalar(&env, "0000000000000000000000000000000000000000000000000000000000000064"));

        let result = client.vrfy_prf(&proof, &inputs);
        assert!(!result, "a real proof must not verify against public inputs it wasn't generated for");
    }
}
