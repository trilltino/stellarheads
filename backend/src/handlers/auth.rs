use sqlx::PgPool;
use crate::services::AuthService;
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use shared::dto::auth::Guest;
use shared::dto::common::ApiResponse;

pub async fn register_guest(
    State(pool): State<PgPool>,
    Json(guest): Json<Guest>,
) -> impl IntoResponse {
    match AuthService::register_or_login_guest(&pool, guest).await {
        Ok(response) => (
            axum::http::StatusCode::OK,
            Json(ApiResponse::success(response, "Authentication successful"))
        ).into_response(),
        Err(err) => err.into_response(),
    }
}