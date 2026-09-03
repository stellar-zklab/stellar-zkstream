#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::StellarAssetClient,
    Address, Bytes, BytesN, Env, Vec,
};

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
        protocol_version: 25,
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
fn test_create_stream_with_cliff_success() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128,
        &1_001_000u64, // start
        &1_010_000u64, // cliff (10s after start)
        &1_100_000u64, // end
        &true,
        &Bytes::new(&env),
        &Vec::new(&env),
    );
    assert_eq!(id, 0u64);

    let s = client.get_stream(&id);
    assert!(s.active);
    assert_eq!(s.cliff_time, 1_010_000u64);
}

#[test]
fn test_cliff_vesting_zero_before_cliff() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128,
        &1_001_000u64, &1_050_000u64, &1_100_000u64, &true,
        &Bytes::new(&env), &Vec::new(&env),
    );

    // Ledger timestamp is 1_000_000 — before start and before cliff
    assert_eq!(client.claimable_amount(&id), 0i128);

    // Set ledger timestamp to 1_020_000 (after start but BEFORE cliff)
    env.ledger().set(LedgerInfo {
        timestamp: 1_020_000,
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });
    // Should still be 0 because cliff timestamp 1_050_000 is not reached
    assert_eq!(client.claimable_amount(&id), 0i128);
}

#[test]
fn test_batch_stream_creation() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

    let rec2 = Address::generate(&env);
    let mut batch = Vec::new(&env);
    batch.push_back(BatchStreamParam {
        recipient: recipient.clone(),
        total_amount: 100_0000000i128,
        start_time: 1_001_000,
        cliff_time: 1_001_000,
        end_time: 1_100_000,
        cancelable: true,
    });
    batch.push_back(BatchStreamParam {
        recipient: rec2.clone(),
        total_amount: 200_0000000i128,
        start_time: 1_001_000,
        cliff_time: 1_001_000,
        end_time: 1_100_000,
        cancelable: false,
    });

    let ids = client.create_batch_streams(&sender, &token, &batch, &Bytes::new(&env), &Vec::new(&env));
    assert_eq!(ids.len(), 2);
    assert_eq!(client.get_stream(&0u64).total_amount, 100_0000000i128);
    assert_eq!(client.get_stream(&1u64).total_amount, 200_0000000i128);
}
