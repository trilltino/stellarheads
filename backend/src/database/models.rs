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
        Self {
            id: 0,
            username,
            wallet_address,
            created_at: Some(Utc::now()),
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
    pub player_result: String,
    pub player_score: i32,
    pub opponent_score: i32,
    pub duration_seconds: f32,
    pub game_mode: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct NewGameInstance {
    pub user_id: Option<i32>,
    pub game_session_id: String,
    pub player_username: String,
    pub player_wallet_address: String,
    pub player_result: String,
    pub player_score: i32,
    pub opponent_score: i32,
    pub duration_seconds: f32,
    pub game_mode: String,
}