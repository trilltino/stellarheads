use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use shared::dto::game::{GameResult, GameResultResponse};
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct GameResultQuery {
    pub username: Option<String>,
    pub wallet_address: Option<String>,
}

pub async fn submit_game_result(
    State(pool): State<PgPool>,
    Json(game_result): Json<GameResult>,
) -> Result<Json<GameResultResponse>, StatusCode> {
    println!("🎮 Received game result for player: {}", game_result.player_username);
    println!("🎯 Match result: {:?}", game_result.match_result);
    println!("📊 Final score: {} - {}", game_result.final_score.left_team, game_result.final_score.right_team);
    println!("⏱️ Duration: {:.2} seconds", game_result.match_duration_seconds);
    
    // For now, we'll just log the result and return success
    // In a real implementation, you might want to store this in a game_results table
    
    // Verify the user exists
    let user_exists = sqlx::query!(
        "SELECT id FROM users WHERE username = $1 AND wallet_address = $2",
        game_result.player_username,
        game_result.player_wallet_address
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        eprintln!("Database error checking user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if user_exists.is_none() {
        return Ok(Json(GameResultResponse {
            success: false,
            message: format!("User {} with wallet address {} not found", 
                           game_result.player_username, 
                           game_result.player_wallet_address),
        }));
    }

    // TODO: Store game result in database
    // For now, just return success
    println!("✅ Game result processed successfully for {}", game_result.player_username);
    
    Ok(Json(GameResultResponse {
        success: true,
        message: format!("Game result recorded for {}", game_result.player_username),
    }))
}

pub async fn get_player_stats(
    State(_pool): State<PgPool>,
    Query(params): Query<GameResultQuery>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {   
    let username = params.username.unwrap_or_else(|| "unknown".to_string());
    let mut stats = HashMap::new();
    
    stats.insert("player".to_string(), username);
    stats.insert("games_played".to_string(), "0".to_string());
    stats.insert("wins".to_string(), "0".to_string());
    stats.insert("losses".to_string(), "0".to_string());
    stats.insert("draws".to_string(), "0".to_string());
    
    println!("📊 Fetched stats for player (placeholder data)");
    
    Ok(Json(stats))
}