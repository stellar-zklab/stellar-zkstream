#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

fn mock_vk(env: &Env) -> Bytes {
    // Layout: alpha(64)+beta(128)+gamma(128)+delta(128)+IC[0](64)+IC[1](64) = 576 bytes
    let mut vk = soroban_sdk::bytes!(env, 0x00);
    vk.append(&Bytes::from_array(env, &[0u8; 575]));
    vk
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_init_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    client.initialize(&admin, &mock_vk(&env));
}

#[test]
fn test_update_vk() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    let new_vk = mock_vk(&env);
    client.update_vk(&admin, &new_vk);
    assert_eq!(client.get_vk(), new_vk);
}
