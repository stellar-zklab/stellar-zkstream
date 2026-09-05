#![no_std]
//! SEP-41 token interface wrapper.
//! Contributors: add allowance-based token helpers here (see issue #13).

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TokenWrapperContract;

#[contractimpl]
impl TokenWrapperContract {
    pub fn version(_env: Env) -> u32 {
        1
    }
}
