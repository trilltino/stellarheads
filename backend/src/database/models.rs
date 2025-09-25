use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub wallet_address: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(username: String, wallet_address: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            username,
            wallet_address,
            created_at: Some(now),
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GameInstance {
    pub id: i32,
    pub user_id: Option<i32>,
    pub game_session_id: String,
    pub player_username: String,
    pub player_wallet_address: String,
    pub player_result: String, // 'Win', 'Loss', 'Draw'
    pub player_score: i32,
    pub opponent_score: i32,
    pub duration_seconds: f32,
    pub game_mode: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl GameInstance {
    pub fn new(
        user_id: Option<i32>,
        game_session_id: String,
        player_username: String,
        player_wallet_address: String,
        player_result: String,
        player_score: i32,
        opponent_score: i32,
        duration_seconds: f32,
        game_mode: String,
    ) -> Self {
        Self {
            id: 0,
            user_id,
            game_session_id,
            player_username,
            player_wallet_address,
            player_result,
            player_score,
            opponent_score,
            duration_seconds,
            game_mode,
            created_at: Some(Utc::now()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub total_games: u32,
    pub avg_duration: f32,
    pub avg_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub username: String,
    pub wallet_address: String,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub total_games: u32,
}