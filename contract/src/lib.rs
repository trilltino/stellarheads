#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, Env, Map, Vec, log, Symbol
};

mod test;

/// Contract errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Player has already joined the leaderboard
    AlreadyJoined = 1,
    /// Player must join the leaderboard before recording wins
    NotJoined = 2,
    /// Invalid input parameter
    InvalidInput = 3,
}

// Data keys for contract storage
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Players,      // Map of all players who have joined
    PlayerWins(Address), // Individual player win counts
}

// Player information stored on the leaderboard
#[derive(Clone)]
#[contracttype]
pub struct Player {
    pub address: Address,
    pub wins: u32,
    pub joined_at: u64, // timestamp when they joined
}

// Leaderboard entry for sorted results
#[derive(Clone)]
#[contracttype]
pub struct LeaderboardEntry {
    pub address: Address,
    pub wins: u32,
}

#[contract]
pub struct StellarHeadsLeaderboard;

#[contractimpl]
impl StellarHeadsLeaderboard {
    /// Join the leaderboard
    /// Players must call this before they can record wins
    /// Returns Ok(()) if successfully joined, Err(Error::AlreadyJoined) if already a member
    pub fn join(env: Env, player: Address) -> Result<(), Error> {
        // Require player to authorize this transaction
        player.require_auth();

        let mut players: Map<Address, Player> = env
            .storage()
            .persistent()
            .get(&DataKey::Players)
            .unwrap_or(Map::new(&env));

        // Check if player already joined
        if players.contains_key(player.clone()) {
            log!(&env, "Player already joined: {}", player);
            return Err(Error::AlreadyJoined);
        }

        // Create new player entry
        let new_player = Player {
            address: player.clone(),
            wins: 0,
            joined_at: env.ledger().timestamp(),
        };

        // Add player to the map
        players.set(player.clone(), new_player);

        // Initialize win count to 0
        env.storage()
            .persistent()
            .set(&DataKey::PlayerWins(player.clone()), &0u32);

        // Save updated players map
        env.storage()
            .persistent()
            .set(&DataKey::Players, &players);

        // Emit event for player join
        env.events().publish(
            (Symbol::new(&env, "player_joined"),),
            player.clone()
        );

        log!(&env, "Player joined leaderboard: {}", player);
        Ok(())
    }

    /// Check if a player has joined the leaderboard
    pub fn has_joined(env: Env, player: Address) -> bool {
        let players: Map<Address, Player> = env
            .storage()
            .persistent()
            .get(&DataKey::Players)
            .unwrap_or(Map::new(&env));

        players.contains_key(player)
    }

    /// Add a win for a player (requires authorization)
    /// Returns Ok(new_win_count) or Err(Error::NotJoined) if player hasn't joined
    pub fn add_win(env: Env, player: Address) -> Result<u32, Error> {
        // Require player to authorize this transaction
        player.require_auth();

        // Check if player has joined
        if !Self::has_joined(env.clone(), player.clone()) {
            return Err(Error::NotJoined);
        }

        // Get current win count
        let current_wins: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::PlayerWins(player.clone()))
            .unwrap_or(0);

        let new_wins = current_wins + 1;

        // Update win count
        env.storage()
            .persistent()
            .set(&DataKey::PlayerWins(player.clone()), &new_wins);

        // Update player in the players map
        let mut players: Map<Address, Player> = env
            .storage()
            .persistent()
            .get(&DataKey::Players)
            .unwrap_or(Map::new(&env));

        if let Some(mut player_data) = players.get(player.clone()) {
            player_data.wins = new_wins;
            players.set(player.clone(), player_data);
            env.storage()
                .persistent()
                .set(&DataKey::Players, &players);
        }

        // Emit event for win addition
        env.events().publish(
            (Symbol::new(&env, "win_added"), player.clone()),
            new_wins
        );

        log!(&env, "Win added for player: {} (total: {})", player, new_wins);
        Ok(new_wins)
    }

    /// Get wins for a specific player
    pub fn get_wins(env: Env, player: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::PlayerWins(player))
            .unwrap_or(0)
    }

    /// Get wins for the calling player (same as get_wins but more explicit)
    pub fn get_my_wins(env: Env, player: Address) -> u32 {
        Self::get_wins(env, player)
    }

    /// Get all players who have joined the leaderboard
    pub fn get_all_players(env: Env) -> Vec<Address> {
        let players: Map<Address, Player> = env
            .storage()
            .persistent()
            .get(&DataKey::Players)
            .unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);
        for (address, _) in players.iter() {
            result.push_back(address);
        }
        result
    }

    /// Get leaderboard sorted by wins (top players first)
    pub fn get_leaderboard(env: Env, limit: u32) -> Vec<LeaderboardEntry> {
        let players: Map<Address, Player> = env
            .storage()
            .persistent()
            .get(&DataKey::Players)
            .unwrap_or(Map::new(&env));

        // Convert to vector for sorting
        let mut entries = Vec::new(&env);
        for (address, player_data) in players.iter() {
            entries.push_back(LeaderboardEntry {
                address,
                wins: player_data.wins,
            });
        }

        // Sort by wins (descending) - manual bubble sort since we can't use std::sort
        let len = entries.len();
        for i in 0..len {
            for j in 0..(len - i - 1) {
                if entries.get(j).unwrap().wins < entries.get(j + 1).unwrap().wins {
                    let temp = entries.get(j).unwrap();
                    entries.set(j, entries.get(j + 1).unwrap());
                    entries.set(j + 1, temp);
                }
            }
        }

        // Apply limit
        let mut result = Vec::new(&env);
        let actual_limit = if limit == 0 || limit > len { len } else { limit };

        for i in 0..actual_limit {
            result.push_back(entries.get(i).unwrap());
        }

        result
    }

    /// Get total number of players
    pub fn get_player_count(env: Env) -> u32 {
        let players: Map<Address, Player> = env
            .storage()
            .persistent()
            .get(&DataKey::Players)
            .unwrap_or(Map::new(&env));

        players.len()
    }

    /// Get player info by address
    pub fn get_player(env: Env, player: Address) -> Option<Player> {
        let players: Map<Address, Player> = env
            .storage()
            .persistent()
            .get(&DataKey::Players)
            .unwrap_or(Map::new(&env));

        players.get(player)
    }
}