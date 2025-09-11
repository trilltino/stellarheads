use axum::{routing::{get, post}, Router};
use crate::handlers::{health, users, auth, soroban};
use crate::database::connection::DbPool;

pub fn create_api_routes() -> Router<DbPool> {
    Router::new()
        .route("/health", get(health))
        .route("/guest", post(auth::register_guest))
        // Soroban/Smart Contract endpoints
        .route("/soroban/contract-info", get(soroban::get_contract_info))
        .route("/soroban/test-signing", post(soroban::test_contract_signing))
        .route("/soroban/submit-score", post(soroban::create_submit_score_transaction))
        .route("/soroban/start-game", post(soroban::create_start_game_transaction))
        .route("/soroban/submit-transaction", post(soroban::submit_signed_transaction))
        .route("/soroban/leaderboard", get(soroban::get_leaderboard))
        .route("/soroban/player-score", get(soroban::get_player_score))
}