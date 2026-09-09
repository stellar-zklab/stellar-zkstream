#![no_std]
//! stellar-zkstream: Privacy-Preserving Payment Streaming Protocol
//! Benchmarked against Sablier V2 specification for Soroban.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Bytes, BytesN, Env, IntoVal, Vec,
};

mod events;
mod storage;

/// How a stream's vested amount grows over time between `cliff_time` and `end_time`.
/// Modeled on Sablier v2's "universal streaming engine" (linear, exponential, and stepped
/// unlocks) — the org's own zkstream README already benchmarks against the Sablier v2 spec.
///
/// `Exponential(exponent)` is a back-loaded curve: vesting starts slow and accelerates,
/// reaching `total_amount` exactly at `end_time` just like `Linear` does, just via a curved
/// path instead of a straight line. `exponent` must be 2, 3, or 4 — kept small deliberately
/// so the fixed-point math in `claimable_internal` never has to raise a scaled value to a
/// power that risks i128 overflow (see the comment there for the actual bound).
///
/// `Stepped(step_count)` vests in `step_count` discrete jumps rather than continuously —
/// e.g. a monthly-cliff vesting schedule with 12 steps, where the recipient's claimable
/// balance jumps once per elapsed step instead of trickling every second.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum VestingCurve {
    Linear,
    Exponential(u32),
    Stepped(u32),
}

#[derive(Clone)]
#[contracttype]
pub struct StreamData {
    pub sender: Address,
    pub recipient: Address,
    pub token: Address,
    pub total_amount: i128,
    pub withdrawn_amount: i128,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
    pub active: bool,
    pub cancelable: bool,
    pub curve: VestingCurve,
}

#[derive(Clone)]
#[contracttype]
pub struct BatchStreamParam {
    pub recipient: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
    pub cancelable: bool,
    pub curve: VestingCurve,
}

/// Validates a curve's own parameters are within the bounds `claimable_internal` relies on
/// to stay overflow-safe. Shared by `create_stream` and `create_batch_streams` so the two
/// paths can't drift into accepting different curve bounds.
fn assert_valid_curve(curve: &VestingCurve) {
    match curve {
        VestingCurve::Linear => {}
        VestingCurve::Exponential(exponent) => {
            assert!(*exponent >= 2 && *exponent <= 4, "exponential curve exponent must be 2..=4");
        }
        VestingCurve::Stepped(step_count) => {
            assert!(*step_count >= 2 && *step_count <= 1000, "stepped curve step_count must be 2..=1000");
        }
    }
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Stream(u64),
    StreamCount,
    Nullifier(BytesN<32>),
    RangeVerifier,
    NullifierVerifier,
    Admin,
    StreamsBySender(Address),
    StreamsByRecipient(Address),
}

#[contract]
pub struct StreamContract;

#[contractimpl]
impl StreamContract {
    /// `range_verifier` and `nullifier_verifier` must be two separate `zk_verifier`
    /// deployments, each initialized with the VK for its own circuit — one `zk_verifier`
    /// instance can't correctly serve both `range_proof` and `nullifier`, since they're
    /// different circuits with different verification keys.
    pub fn initialize(env: Env, admin: Address, range_verifier: Address, nullifier_verifier: Address) {
        if storage::has_admin(&env) {
            panic!("already initialized");
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_range_verifier(&env, &range_verifier);
        storage::set_nullifier_verifier(&env, &nullifier_verifier);
        storage::set_stream_count(&env, 0u64);
    }

    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        total_amount: i128,
        start_time: u64,
        cliff_time: u64,
        end_time: u64,
        cancelable: bool,
        curve: VestingCurve,
        proof: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> u64 {
        sender.require_auth();
        assert!(total_amount > 0, "amount must be positive");
        assert!(end_time > start_time, "end_time must be after start_time");
        assert!(cliff_time >= start_time && cliff_time <= end_time, "invalid cliff_time");
        assert!(start_time >= env.ledger().timestamp(), "start_time in past");
        assert_valid_curve(&curve);

        let verifier = storage::get_range_verifier(&env);
        let args: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![
            &env,
            proof.into_val(&env),
            public_inputs.into_val(&env),
        ];
        let verified: bool = env.invoke_contract(&verifier, &symbol_short!("vrfy_prf"), args);
        assert!(verified, "invalid range proof");

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
            cliff_time,
            end_time,
            active: true,
            cancelable,
            curve,
        };
        storage::set_stream(&env, id, &stream);
        storage::increment_stream_count(&env);
        storage::add_stream_to_sender(&env, &sender, id);
        storage::add_stream_to_recipient(&env, &recipient, id);
        events::emit_stream_created(&env, id, &stream);
        id
    }

    /// Atomic batch stream creation (Sablier V2 feature). Takes one proof and one
    /// public_inputs entry per stream in `streams` (same index) and verifies each against
    /// the range_verifier exactly as create_stream() does for a single stream — batching
    /// the token transfer and storage writes doesn't mean batching away the ZK gating that
    /// every other stream-creation path enforces.
    pub fn create_batch_streams(
        env: Env,
        sender: Address,
        token: Address,
        streams: Vec<BatchStreamParam>,
        proofs: Vec<Bytes>,
        public_inputs: Vec<Vec<BytesN<32>>>,
    ) -> Vec<u64> {
        sender.require_auth();
        assert!(proofs.len() == streams.len(), "one proof required per stream");
        assert!(public_inputs.len() == streams.len(), "one public_inputs entry required per stream");

        let mut created_ids: Vec<u64> = Vec::new(&env);
        let mut total_batch_amount: i128 = 0;
        let verifier = storage::get_range_verifier(&env);

        for i in 0..streams.len() {
            let s = streams.get(i).unwrap();
            assert!(s.total_amount > 0, "amount positive");
            assert!(s.end_time > s.start_time, "end after start");
            assert!(s.cliff_time >= s.start_time && s.cliff_time <= s.end_time, "invalid cliff");
            assert!(s.start_time >= env.ledger().timestamp(), "start_time in past");
            assert_valid_curve(&s.curve);
            total_batch_amount += s.total_amount;

            let args: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![
                &env,
                proofs.get(i).unwrap().into_val(&env),
                public_inputs.get(i).unwrap().into_val(&env),
            ];
            let verified: bool = env.invoke_contract(&verifier, &symbol_short!("vrfy_prf"), args);
            assert!(verified, "invalid range proof");
        }

        soroban_sdk::token::TokenClient::new(&env, &token)
            .transfer(&sender, &env.current_contract_address(), &total_batch_amount);

        for s in streams.iter() {
            let id = storage::get_stream_count(&env);
            let stream = StreamData {
                sender: sender.clone(),
                recipient: s.recipient.clone(),
                token: token.clone(),
                total_amount: s.total_amount,
                withdrawn_amount: 0,
                start_time: s.start_time,
                cliff_time: s.cliff_time,
                end_time: s.end_time,
                active: true,
                cancelable: s.cancelable,
                curve: s.curve.clone(),
            };
            storage::set_stream(&env, id, &stream);
            storage::increment_stream_count(&env);
            storage::add_stream_to_sender(&env, &sender, id);
            storage::add_stream_to_recipient(&env, &s.recipient, id);
            events::emit_stream_created(&env, id, &stream);
            created_ids.push_back(id);
        }
        created_ids
    }

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

        // Bind the caller-supplied `nullifier_hash` (the key used for the spent-nullifier
        // check just below) to the ZK proof's actual public inputs. Without this, the two
        // were independent, unchecked arguments: a caller could submit a valid
        // nullifier_proof/public_inputs pair together with an unrelated, never-before-seen
        // `nullifier_hash`, always pass the "not used" check, and replay the exact same
        // proof indefinitely -- defeating the nullifier's entire double-spend/replay
        // protection. The nullifier circuit's public input layout is
        // `[stream_id, nullifier_hash]` (see circuits/stream_nullifier/nullifier.circom),
        // so this also stops a proof generated for one stream_id being replayed against a
        // different stream this same recipient happens to control.
        assert!(public_inputs.len() == 2, "nullifier public_inputs must be [stream_id, nullifier_hash]");
        let mut expected_stream_id_bytes = [0u8; 32];
        expected_stream_id_bytes[24..32].copy_from_slice(&stream_id.to_be_bytes());
        let expected_stream_id = BytesN::<32>::from_array(&env, &expected_stream_id_bytes);
        assert!(public_inputs.get(0).unwrap() == expected_stream_id, "proof's stream_id doesn't match withdrawal target");
        assert!(public_inputs.get(1).unwrap() == nullifier_hash, "nullifier_hash doesn't match proof's public input");

        assert!(!storage::nullifier_used(&env, &nullifier_hash), "nullifier used");

        let verifier = storage::get_nullifier_verifier(&env);
        let args: soroban_sdk::Vec<soroban_sdk::Val> = soroban_sdk::vec![
            &env,
            nullifier_proof.into_val(&env),
            public_inputs.into_val(&env),
        ];
        let verified: bool = env.invoke_contract(&verifier, &symbol_short!("vrfy_prf"), args);
        assert!(verified, "invalid nullifier proof");

        let now = env.ledger().timestamp();
        let claimable = Self::claimable_internal(&stream, now);
        assert!(claimable > 0, "nothing to withdraw");

        storage::mark_nullifier_used(&env, &nullifier_hash);
        stream.withdrawn_amount += claimable;
        storage::set_stream(&env, stream_id, &stream);

        soroban_sdk::token::TokenClient::new(&env, &stream.token)
            .transfer(&env.current_contract_address(), &caller, &claimable);

        events::emit_withdrawal(&env, stream_id, &caller, claimable);
        claimable
    }

    pub fn cancel_stream(env: Env, stream_id: u64, caller: Address) {
        caller.require_auth();
        let mut stream = storage::get_stream(&env, stream_id);
        assert!(stream.active, "stream inactive");
        assert!(stream.cancelable, "stream non-cancelable");
        assert!(caller == stream.sender, "only sender can cancel");

        let now = env.ledger().timestamp();
        let vested = Self::claimable_internal(&stream, now);
        let remaining = stream.total_amount - stream.withdrawn_amount - vested;

        // Persist the stream as inactive (and record the vested amount as withdrawn)
        // BEFORE making any external token transfer calls below -- the same
        // checks-effects-interactions ordering `withdraw()` already uses, kept here as
        // defense in depth and for consistency. In practice Soroban's own host refuses to
        // let a contract be re-entered while it's still executing ("Contract re-entry is
        // not allowed", confirmed by actually driving a malicious token through this exact
        // path in test.rs), so a reentrant call into cancel_stream never reaches this
        // function's own code at all -- this ordering isn't what's actually stopping that
        // specific attack, the host itself is.
        stream.active = false;
        stream.withdrawn_amount += vested;
        storage::set_stream(&env, stream_id, &stream);

        let token = soroban_sdk::token::TokenClient::new(&env, &stream.token);
        if vested > 0 {
            token.transfer(&env.current_contract_address(), &stream.recipient, &vested);
        }
        if remaining > 0 {
            token.transfer(&env.current_contract_address(), &stream.sender, &remaining);
        }
        events::emit_stream_cancelled(&env, stream_id, &caller);
    }

    /// Transfers the right to withdraw a stream's remaining/future proceeds to a new
    /// address — modeled on Sablier v2's transferable stream NFTs, which make a stream a
    /// tradeable asset (sellable, usable as collateral) rather than a fixed, non-transferable
    /// claim. Only the current recipient can transfer, and only while the stream is still
    /// active (a cancelled stream has nothing left to claim, so transferring it is
    /// meaningless). `withdraw`/`cancel_stream` both re-read `stream.recipient` from storage
    /// on every call, so the new recipient's rights take effect immediately with no other
    /// code changes needed.
    ///
    /// Unlike Sablier, this doesn't require any off-chain secret hand-off: the withdrawal
    /// nullifier (`circuits/stream_nullifier/nullifier.circom`) binds only to
    /// `(secret, stream_id)`, not to any recipient identity — the new recipient picks their
    /// own fresh `secret` and generates their own valid withdrawal proof independently, the
    /// same way the original recipient did.
    pub fn transfer_stream(env: Env, stream_id: u64, caller: Address, new_recipient: Address) {
        caller.require_auth();
        let mut stream = storage::get_stream(&env, stream_id);
        assert!(stream.active, "cannot transfer an inactive stream");
        assert!(caller == stream.recipient, "only the current recipient can transfer this stream");

        let old_recipient = stream.recipient.clone();
        storage::remove_stream_from_recipient(&env, &old_recipient, stream_id);
        storage::add_stream_to_recipient(&env, &new_recipient, stream_id);

        stream.recipient = new_recipient.clone();
        storage::set_stream(&env, stream_id, &stream);

        events::emit_stream_transferred(&env, stream_id, &old_recipient, &new_recipient);
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

    /// Fixed-point scale used for the curved progress fraction below — chosen so that even
    /// the highest-supported exponent (4) keeps every intermediate value comfortably inside
    /// i128 (worst case ~ SCALE^2 = 10^18, far under i128::MAX ~ 1.7 * 10^38).
    const CURVE_SCALE: i128 = 1_000_000_000;

    fn claimable_internal(stream: &StreamData, now: u64) -> i128 {
        if now < stream.start_time || now < stream.cliff_time { return 0; }
        let elapsed = (now.min(stream.end_time) - stream.start_time) as i128;
        let duration = (stream.end_time - stream.start_time) as i128;

        let vested = match &stream.curve {
            VestingCurve::Linear => stream.total_amount * elapsed / duration,
            VestingCurve::Exponential(exponent) => {
                // progress, as a fraction of CURVE_SCALE (0 at start_time, CURVE_SCALE at
                // end_time) — computing this once, then repeatedly squaring/multiplying it
                // by itself and rescaling, keeps every intermediate value near CURVE_SCALE^2
                // at most instead of raising `elapsed` itself to the 4th power (which would
                // overflow for any real-world stream duration).
                let progress = elapsed * Self::CURVE_SCALE / duration;
                let mut curved = progress;
                for _ in 1..*exponent {
                    curved = curved * progress / Self::CURVE_SCALE;
                }
                stream.total_amount * curved / Self::CURVE_SCALE
            }
            VestingCurve::Stepped(step_count) => {
                let step_count = *step_count as i128;
                // Whole steps completed so far — integer division deliberately truncates,
                // so vesting jumps at each step boundary instead of interpolating within one.
                let steps_elapsed = elapsed * step_count / duration;
                stream.total_amount * steps_elapsed / step_count
            }
        };

        (vested - stream.withdrawn_amount).max(0)
    }
}

mod test;
