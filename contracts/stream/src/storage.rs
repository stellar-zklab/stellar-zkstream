use soroban_sdk::{Address, BytesN, Env, Vec};
use crate::DataKey;
use crate::StreamData;

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}
/// `range_proof.circom` and `nullifier.circom` are different circuits with different
/// verification keys, so they need two separate `zk_verifier` deployments — one contract
/// can only hold one VK at a time. See `create_stream`/`withdraw` for which is used where.
pub fn set_range_verifier(env: &Env, v: &Address) {
    env.storage().instance().set(&DataKey::RangeVerifier, v);
}
pub fn get_range_verifier(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::RangeVerifier).expect("not initialized")
}
pub fn set_nullifier_verifier(env: &Env, v: &Address) {
    env.storage().instance().set(&DataKey::NullifierVerifier, v);
}
pub fn get_nullifier_verifier(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::NullifierVerifier).expect("not initialized")
}
pub fn set_stream_count(env: &Env, n: u64) {
    env.storage().instance().set(&DataKey::StreamCount, &n);
}
pub fn get_stream_count(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::StreamCount).unwrap_or(0u64)
}
pub fn increment_stream_count(env: &Env) {
    let n = get_stream_count(env);
    env.storage().instance().set(&DataKey::StreamCount, &(n + 1));
}
pub fn set_stream(env: &Env, id: u64, s: &StreamData) {
    env.storage().persistent().set(&DataKey::Stream(id), s);
}
pub fn get_stream(env: &Env, id: u64) -> StreamData {
    env.storage().persistent().get(&DataKey::Stream(id)).expect("stream not found")
}
pub fn nullifier_used(env: &Env, n: &BytesN<32>) -> bool {
    env.storage().persistent().has(&DataKey::Nullifier(n.clone()))
}
pub fn mark_nullifier_used(env: &Env, n: &BytesN<32>) {
    env.storage().persistent().set(&DataKey::Nullifier(n.clone()), &true);
}
pub fn add_stream_to_sender(env: &Env, sender: &Address, id: u64) {
    let key = DataKey::StreamsBySender(sender.clone());
    let mut ids: Vec<u64> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    ids.push_back(id);
    env.storage().persistent().set(&key, &ids);
}
pub fn get_streams_by_sender(env: &Env, sender: &Address) -> Vec<u64> {
    env.storage().persistent().get(&DataKey::StreamsBySender(sender.clone())).unwrap_or(Vec::new(env))
}
pub fn add_stream_to_recipient(env: &Env, r: &Address, id: u64) {
    let key = DataKey::StreamsByRecipient(r.clone());
    let mut ids: Vec<u64> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    ids.push_back(id);
    env.storage().persistent().set(&key, &ids);
}
pub fn get_streams_by_recipient(env: &Env, r: &Address) -> Vec<u64> {
    env.storage().persistent().get(&DataKey::StreamsByRecipient(r.clone())).unwrap_or(Vec::new(env))
}
