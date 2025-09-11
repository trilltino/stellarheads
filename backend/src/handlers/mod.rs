pub mod auth;
pub mod users;
pub mod soroban;

use axum::http::StatusCode;

pub async fn health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Backend is healthy! 🚀")
}