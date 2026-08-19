#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamData {
    pub id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub total_amount: i128,
    pub withdrawn_amount: i128,
    pub start_time: u64,
    pub end_time: u64,
    pub cliff_time: u64,
    pub is_active: bool,
}

#[contracttype]
pub enum DataKey {
    Stream(u64),
    NextId,
    Admin,
}

#[contract]
pub struct StreamContract;

#[contractimpl]
impl StreamContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextId, &0u64);
    }

    pub fn create_stream(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        start_time: u64,
        end_time: u64,
        cliff_time: u64,
    ) -> u64 {
        sender.require_auth();
        if amount <= 0 {
            panic!("Stream amount must be positive");
        }
        if end_time <= start_time {
            panic!("End time must be after start time");
        }

        let stream_id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(0);
        let stream = StreamData {
            id: stream_id,
            sender: sender.clone(),
            recipient: recipient.clone(),
            total_amount: amount,
            withdrawn_amount: 0,
            start_time,
            end_time,
            cliff_time,
            is_active: true,
        };

        // Persistent storage with explicit Soroban TTL extension (30 days)
        let key = DataKey::Stream(stream_id);
        env.storage().persistent().set(&key, &stream);
        env.storage().persistent().extend_ttl(&key, 172800, 5184000);

        env.storage().instance().set(&DataKey::NextId, &(stream_id + 1));

        env.events().publish(
            (symbol_short!("created"), sender, recipient),
            (stream_id, amount),
        );

        stream_id
    }

    pub fn get_stream(env: Env, stream_id: u64) -> StreamData {
        let key = DataKey::Stream(stream_id);
        if let Some(stream) = env.storage().persistent().get::<DataKey, StreamData>(&key) {
            env.storage().persistent().extend_ttl(&key, 172800, 5184000);
            stream
        } else {
            panic!("Stream not found");
        }
    }
}
