pub mod auth;
pub mod game_results;
pub mod contract;

use axum::{http::StatusCode, Json};
use shared::dto::common::ApiResponse;

pub async fn health() -> (StatusCode, Json<ApiResponse<&'static str>>) {
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Backend is healthy! 🚀",
            "Server is running and database connection is active"
        ))
    )
}