use axum::{
    extract::{Json, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use crate::services::GameService;
use shared::dto::game::{
    StoreGameResultRequest, PlayerStatsQuery,
    PlayerGamesQuery,
};
use shared::dto::common::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RecentGamesQuery {
    pub limit: Option<i64>,
}

#[axum::debug_handler]
pub async fn store_game_result(
    State(pool): State<PgPool>,
    Json(request): Json<StoreGameResultRequest>,
) -> impl IntoResponse {
    match GameService::store_game_result(&pool, request).await {
        Ok(response) => (
            axum::http::StatusCode::OK,
            Json(ApiResponse::success(response, "Game result stored successfully"))
        ).into_response(),
        Err(err) => err.into_response(),
    }
}

pub async fn get_player_stats(
    State(pool): State<PgPool>,
    Query(query): Query<PlayerStatsQuery>,
) -> impl IntoResponse {
    match GameService::get_player_stats(&pool, query).await {
        Ok(stats) => (
            axum::http::StatusCode::OK,
            Json(ApiResponse::success(stats, "Player statistics retrieved successfully"))
        ).into_response(),
        Err(err) => err.into_response(),
    }
}

pub async fn get_player_games(
    State(pool): State<PgPool>,
    Query(query): Query<PlayerGamesQuery>,
) -> impl IntoResponse {
    match GameService::get_player_games(&pool, query).await {
        Ok(games) => (
            axum::http::StatusCode::OK,
            Json(ApiResponse::success(games, "Player games retrieved successfully"))
        ).into_response(),
        Err(err) => err.into_response(),
    }
}

pub async fn get_database_leaderboard(
    State(pool): State<PgPool>,
    Query(query): Query<LeaderboardQuery>,
) -> impl IntoResponse {
    match GameService::get_leaderboard(&pool, query.limit).await {
        Ok(leaderboard) => (
            axum::http::StatusCode::OK,
            Json(ApiResponse::success(leaderboard, "Leaderboard retrieved successfully"))
        ).into_response(),
        Err(err) => err.into_response(),
    }
}

pub async fn get_recent_games(
    State(pool): State<PgPool>,
    Query(query): Query<RecentGamesQuery>,
) -> impl IntoResponse {
    match GameService::get_recent_games(&pool, query.limit).await {
        Ok(games) => (
            axum::http::StatusCode::OK,
            Json(ApiResponse::success(games, "Recent games retrieved successfully"))
        ).into_response(),
        Err(err) => err.into_response(),
    }
}