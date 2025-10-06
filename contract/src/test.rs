#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_join_and_has_joined() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarHeadsLeaderboard);
    let client = StellarHeadsLeaderboardClient::new(&env, &contract_id);

    let player = Address::generate(&env);

    // Player should not be joined initially
    assert!(!client.has_joined(&player));

    // Mock the authorization
    env.mock_all_auths();

    // Join the leaderboard
    let result = client.join(&player);
    assert!(result.is_ok());

    // Player should now be joined
    assert!(client.has_joined(&player));

    // Joining again should return error
    let result_again = client.try_join(&player);
    assert!(result_again.is_err());
}

#[test]
fn test_add_win_and_get_wins() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarHeadsLeaderboard);
    let client = StellarHeadsLeaderboardClient::new(&env, &contract_id);

    let player = Address::generate(&env);

    // Mock the authorization
    env.mock_all_auths();

    // Join first
    client.join(&player);

    // Initial wins should be 0
    assert_eq!(client.get_wins(&player), 0);
    assert_eq!(client.get_my_wins(&player), 0);

    // Add a win
    let new_wins = client.add_win(&player);
    assert_eq!(new_wins, Ok(1));

    // Check wins
    assert_eq!(client.get_wins(&player), 1);
    assert_eq!(client.get_my_wins(&player), 1);

    // Add another win
    let new_wins = client.add_win(&player);
    assert_eq!(new_wins, Ok(2));
    assert_eq!(client.get_wins(&player), 2);
}

#[test]
fn test_leaderboard() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarHeadsLeaderboard);
    let client = StellarHeadsLeaderboardClient::new(&env, &contract_id);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let player3 = Address::generate(&env);

    // Mock the authorization
    env.mock_all_auths();

    // Join all players
    client.join(&player1);
    client.join(&player2);
    client.join(&player3);

    // Add different number of wins
    client.add_win(&player1); // 1 win
    client.add_win(&player2); // 1 win
    client.add_win(&player2); // 2 wins total
    client.add_win(&player3); // 1 win
    client.add_win(&player3); // 2 wins
    client.add_win(&player3); // 3 wins total

    // Check player count
    assert_eq!(client.get_player_count(), 3);

    // Get leaderboard
    let leaderboard = client.get_leaderboard(&10);
    assert_eq!(leaderboard.len(), 3);

    // Should be sorted by wins (descending)
    assert_eq!(leaderboard.get(0).unwrap().wins, 3); // player3
    assert_eq!(leaderboard.get(1).unwrap().wins, 2); // player2
    assert_eq!(leaderboard.get(2).unwrap().wins, 1); // player1

    // Test with limit
    let limited_leaderboard = client.get_leaderboard(&2);
    assert_eq!(limited_leaderboard.len(), 2);
}

#[test]
fn test_get_all_players() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarHeadsLeaderboard);
    let client = StellarHeadsLeaderboardClient::new(&env, &contract_id);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);

    // Mock the authorization
    env.mock_all_auths();

    // Initially no players
    let players = client.get_all_players();
    assert_eq!(players.len(), 0);

    // Join players
    client.join(&player1);
    client.join(&player2);

    // Should have 2 players
    let players = client.get_all_players();
    assert_eq!(players.len(), 2);
}

#[test]
fn test_add_win_without_joining() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StellarHeadsLeaderboard);
    let client = StellarHeadsLeaderboardClient::new(&env, &contract_id);

    let player = Address::generate(&env);

    // Mock the authorization
    env.mock_all_auths();

    // Try to add win without joining first - should return error
    let result = client.try_add_win(&player);
    assert!(result.is_err());
}