use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::database::connection::DbPool;
use crate::database::repositories::game_repository::{GameRepository, PlayerStats, LeaderboardEntry};
use crate::database::repositories::user_repository::UserRepository;
use crate::database::models::GameInstance;

// ================= REQUEST/RESPONSE TYPES =================

#[derive(Debug, Deserialize)]
pub struct StoreGameResultRequest {
    pub game_session_id: String,
    pub player_username: String,
    pub player_wallet_address: String,
    pub player_result: String, // "Win", "Loss", "Draw"
    pub player_score: i32,
    pub opponent_score: i32,
    pub duration_seconds: f32,
    pub game_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StoreGameResultResponse {
    pub success: bool,
    pub message: String,
    pub game_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerStatsQuery {
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct PlayerStatsResponse {
    pub wallet_address: String,
    pub stats: PlayerStats,
}

#[derive(Debug, Deserialize)]
pub struct PlayerGamesQuery {
    pub wallet_address: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PlayerGamesResponse {
    pub wallet_address: String,
    pub games: Vec<GameInstance>,
    pub total_retrieved: usize,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub total_entries: usize,
}

// ================= HANDLER FUNCTIONS =================

/// Store a completed game result in the database
pub async fn store_game_result(
    State(pool): State<DbPool>,
    Json(req): Json<StoreGameResultRequest>,
) -> Result<(StatusCode, Json<StoreGameResultResponse>), (StatusCode, Json<serde_json::Value>)> {
    println!("📊 Storing game result: {:?}", req);

    // Find or create user
    let user = match UserRepository::find_by_wallet_address(&pool, &req.player_wallet_address).await {
        Ok(Some(existing_user)) => Some(existing_user),
        Ok(None) => {
            // Create user if doesn't exist
            match UserRepository::create_guest(&pool, &req.player_username, &req.player_wallet_address).await {
                Ok(new_user) => Some(new_user),
                Err(e) => {
                    println!("⚠️ Failed to create user: {:?}", e);
                    None
                }
            }
        }
        Err(e) => {
            println!("⚠️ Database error finding user: {:?}", e);
            None
        }
    };

    // Create game instance
    let game_instance = GameInstance::new(
        user.as_ref().map(|u| u.id),
        req.game_session_id,
        req.player_username,
        req.player_wallet_address,
        req.player_result,
        req.player_score,
        req.opponent_score,
        req.duration_seconds,
        req.game_mode.unwrap_or_else(|| "single_player_vs_ai".to_string()),
    );

    match GameRepository::create_game_instance(&pool, &game_instance).await {
        Ok(stored_game) => {
            let response = StoreGameResultResponse {
                success: true,
                message: "Game result stored successfully".to_string(),
                game_id: Some(stored_game.id),
            };
            println!("✅ Game result stored with ID: {}", stored_game.id);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to store game result",
                "details": e.to_string()
            });
            println!("❌ Failed to store game result: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get player statistics
pub async fn get_player_stats(
    State(pool): State<DbPool>,
    Query(params): Query<PlayerStatsQuery>,
) -> Result<(StatusCode, Json<PlayerStatsResponse>), (StatusCode, Json<serde_json::Value>)> {
    match GameRepository::get_player_stats(&pool, &params.wallet_address).await {
        Ok(stats) => {
            let response = PlayerStatsResponse {
                wallet_address: params.wallet_address,
                stats,
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to get player stats",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get player's game history
pub async fn get_player_games(
    State(pool): State<DbPool>,
    Query(params): Query<PlayerGamesQuery>,
) -> Result<(StatusCode, Json<PlayerGamesResponse>), (StatusCode, Json<serde_json::Value>)> {
    match GameRepository::get_player_games(&pool, &params.wallet_address, params.limit).await {
        Ok(games) => {
            let response = PlayerGamesResponse {
                wallet_address: params.wallet_address,
                total_retrieved: games.len(),
                games,
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to get player games",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get leaderboard (top players by wins)
pub async fn get_database_leaderboard(
    State(pool): State<DbPool>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<(StatusCode, Json<LeaderboardResponse>), (StatusCode, Json<serde_json::Value>)> {
    match GameRepository::get_leaderboard(&pool, params.limit).await {
        Ok(entries) => {
            let response = LeaderboardResponse {
                total_entries: entries.len(),
                entries,
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to get leaderboard",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get recent games across all players
pub async fn get_recent_games(
    State(pool): State<DbPool>,
    Query(params): Query<LeaderboardQuery>, // Reuse same query params (limit)
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match GameRepository::get_recent_games(&pool, params.limit).await {
        Ok(games) => {
            let response = serde_json::json!({
                "games": games,
                "total_retrieved": games.len(),
                "message": "Recent games retrieved successfully"
            });
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to get recent games",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}