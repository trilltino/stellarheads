pub mod auth;
pub mod leaderboard;
pub mod game_results;

use axum::http::StatusCode;

pub async fn health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Backend is healthy! 🚀")
}