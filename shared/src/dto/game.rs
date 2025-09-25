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

#[derive(Debug, Serialize, Deserialize)]
pub struct GameResultResponse {
    pub success: bool,
    pub message: String,
}