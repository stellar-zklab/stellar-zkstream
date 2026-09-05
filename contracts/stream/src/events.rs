use soroban_sdk::{symbol_short, Address, Env};
use crate::StreamData;

pub fn emit_stream_created(env: &Env, id: u64, s: &StreamData) {
    env.events().publish(
        (symbol_short!("stream"), symbol_short!("created")),
        (id, s.sender.clone(), s.recipient.clone(), s.total_amount),
    );
}
pub fn emit_withdrawal(env: &Env, id: u64, recipient: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("stream"), symbol_short!("withdraw")),
        (id, recipient.clone(), amount),
    );
}
pub fn emit_stream_cancelled(env: &Env, id: u64, sender: &Address) {
    env.events().publish(
        (symbol_short!("stream"), symbol_short!("cancel")),
        (id, sender.clone()),
    );
}
