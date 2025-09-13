use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameResult {
    pub player_username: String,
    pub player_wallet_address: String,
    pub match_result: MatchResult,
    pub final_score: GameScore,
    pub match_duration_seconds: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameScore {
    pub left_team: u32,
    pub right_team: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchResult {
    Win,
    Loss,
    Draw,
}

impl GameResult {
    pub fn new(
        player_username: String,
        player_wallet_address: String,
        match_result: MatchResult,
        final_score: GameScore,
        match_duration_seconds: f32,
    ) -> Self {
        Self {
            player_username,
            player_wallet_address,
            match_result,
            final_score,
            match_duration_seconds,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameResultResponse {
    pub success: bool,
    pub message: String,
}