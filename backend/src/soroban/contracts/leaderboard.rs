#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};


#[contract]
pub struct JoinContract;

#[contractimpl]
impl JoinContract {
    /// Initialize the contract
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&"admin", &admin);
        
        // Initialize empty vector of joined addresses
        let joined_addresses: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&"joined", &joined_addresses);
    }

    /// Join the contract (add address to vector)
    pub fn join(env: Env, player: Address) -> bool {
        player.require_auth();
        
        // Get current list of joined addresses
        let mut joined: Vec<Address> = env.storage()
            .persistent()
            .get(&"joined")
            .unwrap_or_else(|| Vec::new(&env));

        // Check if already joined
        for existing in joined.iter() {
            if existing == player {
                return false; // Already joined
            }
        }

        // Add new address
        joined.push_back(player);
        env.storage().persistent().set(&"joined", &joined);
        
        true
    }

    /// Get all joined addresses
    pub fn get_joined(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&"joined")
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get count of joined addresses
    pub fn get_count(env: Env) -> u32 {
        let joined: Vec<Address> = env.storage()
            .persistent()
            .get(&"joined")
            .unwrap_or_else(|| Vec::new(&env));
        joined.len()
    }

    /// Check if an address has joined
    pub fn has_joined(env: Env, player: Address) -> bool {
        let joined: Vec<Address> = env.storage()
            .persistent()
            .get(&"joined")
            .unwrap_or_else(|| Vec::new(&env));
            
        for existing in joined.iter() {
            if existing == player {
                return true;
            }
        }
        false
    }
}