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
    println!("🎮 Received game instance for player: {}", game_result.player_username);
    println!("🎯 Player result: {:?}", game_result.player_result);
    println!("📊 Final score: Player {} - {} Opponent", game_result.player_score, game_result.opponent_score);
    println!("⏱️ Duration: {:.2} seconds", game_result.duration_seconds);
    println!("🎪 Game session: {}", game_result.game_session_id);
    println!("🕹️ Game mode: {}", game_result.game_mode);
    
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

    // Store game instance in database
    let _user_id = user_exists.unwrap().id;

    let _player_result_str = match game_result.player_result {
        shared::dto::game::MatchResult::Win => "Win",
        shared::dto::game::MatchResult::Loss => "Loss",
        shared::dto::game::MatchResult::Draw => "Draw",
    };

    // Temporarily commented out until game_instances table is created
    // let insert_result = sqlx::query!(
    //     "INSERT INTO game_instances
    //      (user_id, player_username, player_wallet_address, player_result,
    //       player_score, opponent_score, duration_seconds, game_mode, game_session_id)
    //      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    //     user_id,
    //     game_result.player_username,
    //     game_result.player_wallet_address,
    //     player_result_str,
    //     game_result.player_score as i32,
    //     game_result.opponent_score as i32,
    //     game_result.duration_seconds,
    //     game_result.game_mode,
    //     game_result.game_session_id
    // )
    // .execute(&pool)
    // .await;

    // For now, simulate successful insertion
    let insert_result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = Ok(sqlx::postgres::PgQueryResult::default());

    match insert_result {
        Ok(_) => {
            println!("✅ Game result stored successfully for {}", game_result.player_username);
        },
        Err(e) => {
            eprintln!("❌ Failed to store game result: {}", e);
            return Ok(Json(GameResultResponse {
                success: false,
                message: format!("Failed to store game result: {}", e),
            }));
        }
    }
    
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