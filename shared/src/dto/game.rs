use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameResult {
    pub player_username: String,
    pub player_wallet_address: String,
    pub player_result: MatchResult,
    pub player_score: u32,
    pub opponent_score: u32,
    pub game_session_id: String,
    pub game_mode: String,
    pub duration_seconds: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GameScore {
    pub left_team: u32,
    pub right_team: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchResult {
    Win,
    Loss,
    Draw,
}

impl std::fmt::Display for MatchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchResult::Win => write!(f, "Win"),
            MatchResult::Loss => write!(f, "Loss"),
            MatchResult::Draw => write!(f, "Draw"),
        }
    }
}

impl GameResult {
    pub fn new(
        player_username: String,
        player_wallet_address: String,
        player_result: MatchResult,
        player_score: u32,
        opponent_score: u32,
        duration_seconds: f32,
        game_session_id: String,
    ) -> Self {
        Self {
            player_username,
            player_wallet_address,
            player_result,
            player_score,
            opponent_score,
            game_session_id,
            game_mode: "single_player".to_string(),
            duration_seconds,
            timestamp: Utc::now(),
        }
    }

    pub fn with_game_mode(mut self, game_mode: String) -> Self {
        self.game_mode = game_mode;
        self
    }
}

// API Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct StoreGameResultRequest {
    pub game_session_id: String,
    pub player_username: String,
    pub player_wallet_address: String,
    pub player_result: String,
    pub player_score: i32,
    pub opponent_score: i32,
    pub duration_seconds: f32,
    pub game_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StoreGameResultResponse {
    pub game_id: i32,
    pub contract_xdr: Option<ContractXdrInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractXdrInfo {
    pub xdr: String,
    pub function_name: String,
    pub description: String,
    pub network_passphrase: String,
}

#[derive(Debug, Deserialize)]
pub struct PlayerStatsQuery {
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct PlayerStats {
    pub total_games: i64,
    pub wins: i64,
    pub losses: i64,
    pub draws: i64,
    pub win_rate: f64,
    pub average_score: f64,
    pub best_score: i32,
    pub total_playtime_seconds: f64,
}

#[derive(Debug, Deserialize)]
pub struct PlayerGamesQuery {
    pub wallet_address: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GameInstance {
    pub id: i32,
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

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub username: String,
    pub wallet_address: String,
    pub wins: i64,
    pub total_games: i64,
    pub win_rate: f64,
    pub best_score: i32,
}