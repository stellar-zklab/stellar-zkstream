#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Bytes, BytesN, Env, Vec,
};

mod events;
mod storage;

/// Data stored on-chain per payment stream.
#[derive(Clone)]
#[contracttype]
pub struct StreamData {
    pub sender: Address,
    pub recipient: Address,
    pub token: Address,
    pub total_amount: i128,
    pub withdrawn_amount: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Stream(u64),
    StreamCount,
    Nullifier(BytesN<32>),
    VerifierContract,
    Admin,
    StreamsBySender(Address),
    StreamsByRecipient(Address),
}

#[contract]
pub struct StreamContract;

#[contractimpl]
impl StreamContract {
    /// Initialize with admin address and ZK verifier contract address.
    pub fn initialize(env: Env, admin: Address, verifier_contract: Address) {
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_verifier(&env, &verifier_contract);
        storage::set_stream_count(&env, 0u64);
    }

    /// Create a payment stream gated by a ZK range proof.
    /// Tokens are escrowed in this contract immediately.
    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        total_amount: i128,
        start_time: u64,
        end_time: u64,
        proof: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> u64 {
        sender.require_auth();
        assert!(total_amount > 0, "amount must be positive");
        assert!(end_time > start_time, "end_time must be after start_time");
        assert!(start_time >= env.ledger().timestamp(), "start_time in past");

        // Verify ZK range proof via verifier contract
        let verifier = storage::get_verifier(&env);
        let verified: bool = env.invoke_contract(
            &verifier,
            &symbol_short!("vrfy_prf"),
            soroban_sdk::vec![&env, proof.into_val(&env), public_inputs.into_val(&env)],
        );
        assert!(verified, "invalid range proof");

        // Escrow tokens
        soroban_sdk::token::TokenClient::new(&env, &token)
            .transfer(&sender, &env.current_contract_address(), &total_amount);

        let id = storage::get_stream_count(&env);
        let stream = StreamData {
            sender: sender.clone(),
            recipient: recipient.clone(),
            token,
            total_amount,
            withdrawn_amount: 0,
            start_time,
            end_time,
            active: true,
        };
        storage::set_stream(&env, id, &stream);
        storage::increment_stream_count(&env);
        storage::add_stream_to_sender(&env, &sender, id);
        storage::add_stream_to_recipient(&env, &recipient, id);
        events::emit_stream_created(&env, id, &stream);
        id
    }

    /// Withdraw vested tokens. Requires a ZK nullifier proof to prevent double-spend.
    pub fn withdraw(
        env: Env,
        stream_id: u64,
        caller: Address,
        nullifier_hash: BytesN<32>,
        nullifier_proof: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> i128 {
        caller.require_auth();
        let mut stream = storage::get_stream(&env, stream_id);
        assert!(stream.active, "stream not active");
        assert!(caller == stream.recipient, "only recipient can withdraw");
        assert!(!storage::nullifier_used(&env, &nullifier_hash), "nullifier already used");

        // Verify nullifier proof
        let verifier = storage::get_verifier(&env);
        let verified: bool = env.invoke_contract(
            &verifier,
            &symbol_short!("vrfy_prf"),
            soroban_sdk::vec![&env, nullifier_proof.into_val(&env), public_inputs.into_val(&env)],
        );
        assert!(verified, "invalid nullifier proof");

        let now = env.ledger().timestamp();
        let claimable = Self::claimable_internal(&stream, now);
        assert!(claimable > 0, "nothing to withdraw yet");

        storage::mark_nullifier_used(&env, &nullifier_hash);
        stream.withdrawn_amount += claimable;
        storage::set_stream(&env, stream_id, &stream);

        soroban_sdk::token::TokenClient::new(&env, &stream.token)
            .transfer(&env.current_contract_address(), &caller, &claimable);

        events::emit_withdrawal(&env, stream_id, &caller, claimable);
        claimable
    }

    /// Cancel a stream. Sender gets unvested tokens, recipient gets any vested amount.
    pub fn cancel_stream(env: Env, stream_id: u64, caller: Address) {
        caller.require_auth();
        let mut stream = storage::get_stream(&env, stream_id);
        assert!(stream.active, "stream already inactive");
        assert!(caller == stream.sender, "only sender can cancel");

        let now = env.ledger().timestamp();
        let vested = Self::claimable_internal(&stream, now);
        let token_client = soroban_sdk::token::TokenClient::new(&env, &stream.token);

        if vested > 0 {
            token_client.transfer(&env.current_contract_address(), &stream.recipient, &vested);
        }
        let remaining = stream.total_amount - stream.withdrawn_amount - vested;
        if remaining > 0 {
            token_client.transfer(&env.current_contract_address(), &stream.sender, &remaining);
        }

        stream.active = false;
        stream.withdrawn_amount += vested;
        storage::set_stream(&env, stream_id, &stream);
        events::emit_stream_cancelled(&env, stream_id, &caller);
    }

    pub fn get_stream(env: Env, stream_id: u64) -> StreamData {
        storage::get_stream(&env, stream_id)
    }

    pub fn get_streams_by_sender(env: Env, sender: Address) -> Vec<u64> {
        storage::get_streams_by_sender(&env, &sender)
    }

    pub fn get_streams_by_recipient(env: Env, recipient: Address) -> Vec<u64> {
        storage::get_streams_by_recipient(&env, &recipient)
    }

    pub fn claimable_amount(env: Env, stream_id: u64) -> i128 {
        let stream = storage::get_stream(&env, stream_id);
        Self::claimable_internal(&stream, env.ledger().timestamp())
    }

    fn claimable_internal(stream: &StreamData, now: u64) -> i128 {
        if now < stream.start_time { return 0; }
        let elapsed = (now.min(stream.end_time) - stream.start_time) as i128;
        let duration = (stream.end_time - stream.start_time) as i128;
        let vested = stream.total_amount * elapsed / duration;
        (vested - stream.withdrawn_amount).max(0)
    }
}

mod test;
