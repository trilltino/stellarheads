use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameResult {
    // Player identification
    pub player_username: String,
    pub player_wallet_address: String,

    // Game outcome for this specific player
    pub player_result: MatchResult, // Win/Loss/Draw from player's perspective

    // Scores
    pub player_score: u32,    // This player's score
    pub opponent_score: u32,  // AI or opponent score

    // Game session info
    pub game_session_id: String, // UUID for this game instance (required)
    pub game_mode: String,    // "single_player", "multiplayer", etc.

    // Performance metrics
    pub duration_seconds: f32,

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
        player_result: MatchResult,
        player_score: u32,
        opponent_score: u32,
        duration_seconds: f32,
        game_session_id: String, // Require game session ID to be passed in
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