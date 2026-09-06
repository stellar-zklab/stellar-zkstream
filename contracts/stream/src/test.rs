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

#[contract]
struct RejectingVerifier;

#[contractimpl]
impl RejectingVerifier {
    pub fn vrfy_prf(_env: Env, _proof: Bytes, _inputs: Vec<BytesN<32>>) -> bool {
        false
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

    let proofs = Vec::from_array(&env, [Bytes::new(&env), Bytes::new(&env)]);
    let public_inputs = Vec::from_array(&env, [Vec::new(&env), Vec::new(&env)]);
    let ids = client.create_batch_streams(&sender, &token, &batch, &proofs, &public_inputs);
    assert_eq!(ids.len(), 2);
    assert_eq!(client.get_stream(&0u64).total_amount, 100_0000000i128);
    assert_eq!(client.get_stream(&1u64).total_amount, 200_0000000i128);
}

#[test]
#[should_panic(expected = "one proof required per stream")]
fn test_batch_stream_creation_rejects_mismatched_proof_count() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

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
        recipient: recipient.clone(),
        total_amount: 100_0000000i128,
        start_time: 1_001_000,
        cliff_time: 1_001_000,
        end_time: 1_100_000,
        cancelable: true,
    });

    // Two streams, but only one proof supplied — must be rejected outright rather than
    // silently verifying the first stream and skipping the second.
    let proofs = Vec::from_array(&env, [Bytes::new(&env)]);
    let public_inputs = Vec::from_array(&env, [Vec::new(&env)]);
    client.create_batch_streams(&sender, &token, &batch, &proofs, &public_inputs);
}

/// A "token" whose `transfer` calls back into `cancel_stream` on its first invocation
/// after being armed. Used to empirically confirm whether `cancel_stream` is vulnerable
/// to reentrancy (its token.transfer() calls happen before the stream is marked inactive
/// / withdrawn_amount is persisted, unlike `withdraw`, which updates storage first).
#[contract]
struct ReentrantToken;

#[contractimpl]
impl ReentrantToken {
    pub fn arm(env: Env, stream_contract: Address, stream_id: u64, canceller: Address) {
        env.storage().instance().set(&symbol_short!("sc"), &stream_contract);
        env.storage().instance().set(&symbol_short!("sid"), &stream_id);
        env.storage().instance().set(&symbol_short!("who"), &canceller);
        env.storage().instance().set(&symbol_short!("armed"), &true);
    }

    pub fn transfer(env: Env, _from: Address, _to: Address, _amount: i128) {
        let n: u32 = env.storage().instance().get(&symbol_short!("xfers")).unwrap_or(0);
        env.storage().instance().set(&symbol_short!("xfers"), &(n + 1));

        let armed: bool = env.storage().instance().get(&symbol_short!("armed")).unwrap_or(false);
        let already: bool = env.storage().instance().get(&symbol_short!("rentrd")).unwrap_or(false);
        if armed && !already {
            env.storage().instance().set(&symbol_short!("rentrd"), &true);
            let sc: Address = env.storage().instance().get(&symbol_short!("sc")).unwrap();
            let sid: u64 = env.storage().instance().get(&symbol_short!("sid")).unwrap();
            let who: Address = env.storage().instance().get(&symbol_short!("who")).unwrap();
            let client = StreamContractClient::new(&env, &sc);
            client.cancel_stream(&sid, &who);
        }
    }

    pub fn transfer_count(env: Env) -> u32 {
        env.storage().instance().get(&symbol_short!("xfers")).unwrap_or(0)
    }
}

#[test]
#[should_panic(expected = "stream inactive")]
fn test_cancel_stream_reentrancy_is_blocked_by_effects_before_interactions() {
    // Uses a malicious token in place of the real SEP-41 asset contract to confirm, by
    // actually running it, that cancel_stream can no longer be reentered to pay out the
    // same vested/unvested split twice. cancel_stream now persists `stream.active = false`
    // and `withdrawn_amount` BEFORE making any token.transfer() call (matching the
    // checks-effects-interactions ordering withdraw() already used), so the token's
    // reentrant call back into cancel_stream sees the stream already inactive and panics
    // -- which aborts the whole transaction rather than allowing a double payout.
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

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let verifier = env.register(MockVerifier, ());
    let evil_token = env.register(ReentrantToken, ());

    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

    let id = client.create_stream(
        &sender, &recipient, &evil_token,
        &1_000_0000000i128,
        &1_001_000u64, &1_001_000u64, &1_100_000u64, &true, // no cliff, cancelable
        &Bytes::new(&env), &Vec::new(&env),
    );

    // Halfway through the stream: half is vested, half remains unvested.
    env.ledger().set(LedgerInfo {
        timestamp: 1_050_500,
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let evil_client = ReentrantTokenClient::new(&env, &evil_token);
    evil_client.arm(&cid, &id, &sender);

    // Sender cancels. The token's transfer() attempts to reenter cancel_stream; that
    // nested call must now fail instead of paying out the vested/unvested split again.
    client.cancel_stream(&id, &sender);
}

/// Builds the `[stream_id, nullifier_hash]` public_inputs layout the nullifier circuit
/// produces (see circuits/stream_nullifier/nullifier.circom) and that withdraw() now
/// requires to match its `stream_id`/`nullifier_hash` arguments.
fn nullifier_public_inputs(env: &Env, stream_id: u64, nullifier_hash: &BytesN<32>) -> Vec<BytesN<32>> {
    let mut stream_id_bytes = [0u8; 32];
    stream_id_bytes[24..32].copy_from_slice(&stream_id.to_be_bytes());
    let stream_id_input = BytesN::from_array(env, &stream_id_bytes);
    Vec::from_array(env, [stream_id_input, nullifier_hash.clone()])
}

#[test]
fn test_withdraw_success_with_bound_nullifier() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128,
        &1_001_000u64, &1_001_000u64, &1_100_000u64, &true,
        &Bytes::new(&env), &Vec::new(&env),
    );

    env.ledger().set(LedgerInfo {
        timestamp: 1_050_500, // halfway vested
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let nullifier_hash = BytesN::from_array(&env, &[7u8; 32]);
    let public_inputs = nullifier_public_inputs(&env, id, &nullifier_hash);

    let claimed = client.withdraw(&id, &recipient, &nullifier_hash, &Bytes::new(&env), &public_inputs);
    assert_eq!(claimed, 500_0000000i128);
    assert_eq!(client.get_stream(&id).withdrawn_amount, 500_0000000i128);
}

#[test]
#[should_panic(expected = "nullifier used")]
fn test_withdraw_rejects_a_replayed_nullifier() {
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128,
        &1_001_000u64, &1_001_000u64, &1_100_000u64, &true,
        &Bytes::new(&env), &Vec::new(&env),
    );

    env.ledger().set(LedgerInfo {
        timestamp: 1_050_500,
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let nullifier_hash = BytesN::from_array(&env, &[7u8; 32]);
    let public_inputs = nullifier_public_inputs(&env, id, &nullifier_hash);
    client.withdraw(&id, &recipient, &nullifier_hash, &Bytes::new(&env), &public_inputs);

    // Replaying the exact same nullifier_hash/public_inputs a second time must fail.
    client.withdraw(&id, &recipient, &nullifier_hash, &Bytes::new(&env), &public_inputs);
}

#[test]
#[should_panic(expected = "nullifier_hash doesn't match proof's public input")]
fn test_withdraw_rejects_nullifier_hash_not_bound_to_public_inputs() {
    // Pins the fix for the vulnerability this repo actually shipped with: withdraw()
    // used to track "used" nullifiers by a caller-supplied `nullifier_hash` argument
    // that was never checked against the ZK proof's own `public_inputs`. That meant a
    // caller could submit a valid nullifier_proof/public_inputs pair together with an
    // arbitrary, never-before-seen `nullifier_hash`, always pass the "not used" check,
    // and replay the exact same proof indefinitely. This test picks a `nullifier_hash`
    // that deliberately does not match public_inputs[1] and confirms it's now rejected.
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128,
        &1_001_000u64, &1_001_000u64, &1_100_000u64, &true,
        &Bytes::new(&env), &Vec::new(&env),
    );

    env.ledger().set(LedgerInfo {
        timestamp: 1_050_500,
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let real_nullifier = BytesN::from_array(&env, &[7u8; 32]);
    let public_inputs = nullifier_public_inputs(&env, id, &real_nullifier);
    let arbitrary_nullifier = BytesN::from_array(&env, &[9u8; 32]);

    client.withdraw(&id, &recipient, &arbitrary_nullifier, &Bytes::new(&env), &public_inputs);
}

#[test]
fn test_cancel_stream_splits_vested_and_unvested_correctly() {
    // cancel_stream is claimed as "tested" in README.md but had zero test coverage
    // before this audit. Confirms the sender gets back exactly the unvested remainder
    // and the recipient gets exactly what had already vested -- no more, no less.
    let (env, token, sender, recipient, verifier) = setup();
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &verifier, &verifier);

    let id = client.create_stream(
        &sender, &recipient, &token,
        &1_000_0000000i128,
        &1_001_000u64, &1_001_000u64, &1_100_000u64, &true,
        &Bytes::new(&env), &Vec::new(&env),
    );

    env.ledger().set(LedgerInfo {
        timestamp: 1_050_500, // halfway vested
        protocol_version: 25,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let recipient_before = soroban_sdk::token::TokenClient::new(&env, &token).balance(&recipient);
    let sender_before = soroban_sdk::token::TokenClient::new(&env, &token).balance(&sender);

    client.cancel_stream(&id, &sender);

    let token_client = soroban_sdk::token::TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&recipient) - recipient_before, 500_0000000i128);
    assert_eq!(token_client.balance(&sender) - sender_before, 500_0000000i128);

    let s = client.get_stream(&id);
    assert!(!s.active);
    assert_eq!(s.withdrawn_amount, 500_0000000i128);
}

#[test]
#[should_panic(expected = "invalid range proof")]
fn test_batch_stream_creation_actually_calls_the_verifier_and_rejects_a_failing_proof() {
    let (env, token, sender, recipient, _verifier) = setup();
    // A verifier that always rejects — if create_batch_streams still discarded its proof
    // arguments (the bug this fix closes), this stream would be created anyway.
    let rejecting_verifier = env.register(RejectingVerifier, ());
    let cid = env.register(StreamContract, ());
    let client = StreamContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &rejecting_verifier, &rejecting_verifier);

    let mut batch = Vec::new(&env);
    batch.push_back(BatchStreamParam {
        recipient,
        total_amount: 100_0000000i128,
        start_time: 1_001_000,
        cliff_time: 1_001_000,
        end_time: 1_100_000,
        cancelable: true,
    });

    let proofs = Vec::from_array(&env, [Bytes::new(&env)]);
    let public_inputs = Vec::from_array(&env, [Vec::new(&env)]);
    client.create_batch_streams(&sender, &token, &batch, &proofs, &public_inputs);
}
