#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::{StellarAssetClient},
    Address, Bytes, BytesN, Env, Vec,
};

#[contract]
struct MockVerifier;
#[contractimpl]
impl MockVerifier {
    pub fn vrfy_prf(_env: Env, _proof: Bytes, _inputs: Vec<BytesN<32>>) -> bool { true }
}

fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo { timestamp: 1_000_000, ..Default::default() });
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    StellarAssetClient::new(&env, &token).mint(&Address::generate(&env), &0);
    let sender = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&sender, &10_000_0000000);
    let recipient = Address::generate(&env);
    let verifier = env.register(MockVerifier, ());
    (env, token, sender, recipient, verifier)
}

#[test]
fn test_create_stream() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier);
    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128, &1_001_000u64, &1_100_000u64,
        &Bytes::new(&env), &Vec::new(&env),
    );
    assert_eq!(id, 0u64);
    let s = client.get_stream(&0u64);
    assert!(s.active);
    assert_eq!(s.total_amount, 1_000_0000000);
}

#[test]
fn test_cancel_stream() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier);
    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128, &1_001_000u64, &1_100_000u64,
        &Bytes::new(&env), &Vec::new(&env),
    );
    client.cancel_stream(&id, &sender);
    assert!(!client.get_stream(&id).active);
}

#[test]
fn test_streams_by_sender() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier);
    client.create_stream(&sender, &recipient, &token, &100_0000000i128, &1_001_000u64, &1_100_000u64, &Bytes::new(&env), &Vec::new(&env));
    client.create_stream(&sender, &recipient, &token, &200_0000000i128, &1_001_000u64, &1_200_000u64, &Bytes::new(&env), &Vec::new(&env));
    assert_eq!(client.get_streams_by_sender(&sender).len(), 2);
}
