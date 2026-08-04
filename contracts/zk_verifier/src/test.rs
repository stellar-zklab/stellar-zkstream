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
fn test_verify_proof_with_valid_structure() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));

    let proof = mock_proof(&env);
    let inputs: Vec<BytesN<32>> = Vec::new(&env);
    // With correctly sized proof+vk, verify returns true (stub — full impl in issue #1)
    let result = client.vrfy_prf(&proof, &inputs);
    assert!(result);
}
