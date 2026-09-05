#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, Vec};

mod groth16;

#[derive(Clone)]
#[contracttype]
pub enum DataKey { Admin, VerificationKey, Initialized }

#[contract]
pub struct ZkVerifierContract;

#[contractimpl]
impl ZkVerifierContract {
    /// Initialize with admin and a serialized Groth16 BN254 verification key.
    /// The VK is produced by the circom2soroban tool from snarkjs output.
    pub fn initialize(env: Env, admin: Address, verification_key: Bytes) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::VerificationKey, &verification_key);
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    /// Verify a Groth16 BN254 proof for real, using Soroban Protocol 25's native
    /// `crypto().bn254()` host functions (g1_add, g1_mul, pairing_check — point negation
    /// is plain field arithmetic done client-side, no separate host call needed). See
    /// `groth16::verify` for the actual pairing-equation implementation.
    pub fn vrfy_prf(env: Env, proof: Bytes, public_inputs: Vec<BytesN<32>>) -> bool {
        let vk: Bytes = env.storage().instance()
            .get(&DataKey::VerificationKey).expect("not initialized");
        groth16::verify(&env, &vk, &proof, &public_inputs)
    }

    /// Upgrade the verification key (admin only). Use when circuit changes.
    pub fn update_vk(env: Env, admin: Address, new_key: Bytes) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        assert!(admin == stored, "unauthorized");
        env.storage().instance().set(&DataKey::VerificationKey, &new_key);
    }

    pub fn get_vk(env: Env) -> Bytes {
        env.storage().instance().get(&DataKey::VerificationKey).expect("not initialized")
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).expect("not initialized")
    }
}

mod test;
