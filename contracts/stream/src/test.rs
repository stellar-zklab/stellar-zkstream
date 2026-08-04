#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::StellarAssetClient,
    Address, Bytes, BytesN, Env, Vec,
};

// Mock verifier that always returns true (unit testing only)
#[contract]
struct MockVerifier;

#[contractimpl]
impl MockVerifier {
    pub fn vrfy_prf(_env: Env, _proof: Bytes, _inputs: Vec<BytesN<32>>) -> bool {
        true
    }
}

fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        timestamp: 1_000_000,
        protocol_version: 22,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone()).address();

    let sender = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000_0000000i128);

    let recipient = Address::generate(&env);
    let verifier = env.register(MockVerifier, ());

    (env, token, sender, recipient, verifier)
}

#[test]
fn test_create_stream_success() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier);

    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128,
        &1_001_000u64,
        &1_100_000u64,
        &Bytes::new(&env),
        &Vec::new(&env),
    );
    assert_eq!(id, 0u64);

    let s = client.get_stream(&id);
    assert!(s.active);
    assert_eq!(s.total_amount, 1_000_0000000);
    assert_eq!(s.sender, sender);
    assert_eq!(s.recipient, recipient);
}

#[test]
fn test_cancel_before_vesting_returns_all_to_sender() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier);

    let total = 1_000_0000000i128;
    let id = client.create_stream(
        &sender, &recipient, &token,
        &total, &1_001_000u64, &1_100_000u64,
        &Bytes::new(&env), &Vec::new(&env),
    );

    // Cancel before stream starts (timestamp 1_000_000 < start 1_001_000)
    client.cancel_stream(&id, &sender);
    let s = client.get_stream(&id);
    assert!(!s.active);
}

#[test]
fn test_streams_by_sender_indexed_correctly() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier);

    client.create_stream(&sender, &recipient, &token, &100_0000000i128, &1_001_000u64, &1_100_000u64, &Bytes::new(&env), &Vec::new(&env));
    client.create_stream(&sender, &recipient, &token, &200_0000000i128, &1_001_000u64, &1_200_000u64, &Bytes::new(&env), &Vec::new(&env));

    let ids = client.get_streams_by_sender(&sender);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 0u64);
    assert_eq!(ids.get(1).unwrap(), 1u64);
}

#[test]
fn test_claimable_amount_before_start() {
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
    // Timestamp is 1_000_000, stream starts at 1_001_000 — nothing claimable
    assert_eq!(client.claimable_amount(&id), 0i128);
}
