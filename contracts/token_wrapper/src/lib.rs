#![no_std]
// SEP-41 token interface wrapper — re-exports standard token client for use across contracts.
// Contributors: implement allowance-based token helpers here (issue #13).
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TokenWrapperContract;

#[contractimpl]
impl TokenWrapperContract {
    pub fn version(_env: Env) -> u32 { 1 }
}
