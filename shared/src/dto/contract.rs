use serde::{Deserialize, Serialize};

/// Contract function types that mirror the smart contract functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeaderboardFunction {
    /// join(player: Address) -> bool
    Join { player: String },
    /// has_joined(player: Address) -> bool
    HasJoined { player: String },
    /// add_win(player: Address) -> u32
    AddWin { player: String },
    /// get_wins(player: Address) -> u32
    GetWins { player: String },
    /// get_my_wins(player: Address) -> u32
    GetMyWins { player: String },
    /// get_all_players() -> Vec<Address>
    GetAllPlayers,
    /// get_leaderboard(limit: u32) -> Vec<LeaderboardEntry>
    GetLeaderboard { limit: u32 },
    /// get_player_count() -> u32
    GetPlayerCount,
    /// get_player(player: Address) -> Option<Player>
    GetPlayer { player: String },
}

impl LeaderboardFunction {
    /// Get the function name as it appears in the contract
    pub fn name(&self) -> &'static str {
        match self {
            LeaderboardFunction::Join { .. } => "join",
            LeaderboardFunction::HasJoined { .. } => "has_joined",
            LeaderboardFunction::AddWin { .. } => "add_win",
            LeaderboardFunction::GetWins { .. } => "get_wins",
            LeaderboardFunction::GetMyWins { .. } => "get_my_wins",
            LeaderboardFunction::GetAllPlayers => "get_all_players",
            LeaderboardFunction::GetLeaderboard { .. } => "get_leaderboard",
            LeaderboardFunction::GetPlayerCount => "get_player_count",
            LeaderboardFunction::GetPlayer { .. } => "get_player",
        }
    }

    /// Get the function signature for display
    pub fn signature(&self) -> &'static str {
        match self {
            LeaderboardFunction::Join { .. } => "join(player: Address) -> bool",
            LeaderboardFunction::HasJoined { .. } => "has_joined(player: Address) -> bool",
            LeaderboardFunction::AddWin { .. } => "add_win(player: Address) -> u32",
            LeaderboardFunction::GetWins { .. } => "get_wins(player: Address) -> u32",
            LeaderboardFunction::GetMyWins { .. } => "get_my_wins(player: Address) -> u32",
            LeaderboardFunction::GetAllPlayers => "get_all_players() -> Vec<Address>",
            LeaderboardFunction::GetLeaderboard { .. } => "get_leaderboard(limit: u32) -> Vec<LeaderboardEntry>",
            LeaderboardFunction::GetPlayerCount => "get_player_count() -> u32",
            LeaderboardFunction::GetPlayer { .. } => "get_player(player: Address) -> Option<Player>",
        }
    }

    /// Get function description
    pub fn description(&self) -> &'static str {
        match self {
            LeaderboardFunction::Join { .. } => "Join the leaderboard to start tracking wins",
            LeaderboardFunction::HasJoined { .. } => "Check if a player has joined the leaderboard",
            LeaderboardFunction::AddWin { .. } => "Add a win for the player (requires authorization)",
            LeaderboardFunction::GetWins { .. } => "Get total wins for a specific player",
            LeaderboardFunction::GetMyWins { .. } => "Get wins for the calling player",
            LeaderboardFunction::GetAllPlayers => "Get all players who have joined",
            LeaderboardFunction::GetLeaderboard { .. } => "Get leaderboard sorted by wins",
            LeaderboardFunction::GetPlayerCount => "Get total number of players",
            LeaderboardFunction::GetPlayer { .. } => "Get detailed player information",
        }
    }

    /// Get the display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            LeaderboardFunction::Join { .. } => "Join Leaderboard",
            LeaderboardFunction::HasJoined { .. } => "Check Joined",
            LeaderboardFunction::AddWin { .. } => "Add Win",
            LeaderboardFunction::GetWins { .. } => "Get Wins",
            LeaderboardFunction::GetMyWins { .. } => "Get My Wins",
            LeaderboardFunction::GetAllPlayers => "Get All Players",
            LeaderboardFunction::GetLeaderboard { .. } => "Get Leaderboard",
            LeaderboardFunction::GetPlayerCount => "Get Player Count",
            LeaderboardFunction::GetPlayer { .. } => "Get Player",
        }
    }

    /// Get the icon for the function
    pub fn icon(&self) -> &'static str {
        match self {
            LeaderboardFunction::Join { .. } => "🎯",
            LeaderboardFunction::HasJoined { .. } => "❓",
            LeaderboardFunction::AddWin { .. } => "🏆",
            LeaderboardFunction::GetWins { .. } => "📊",
            LeaderboardFunction::GetMyWins { .. } => "🏅",
            LeaderboardFunction::GetAllPlayers => "👥",
            LeaderboardFunction::GetLeaderboard { .. } => "🏆",
            LeaderboardFunction::GetPlayerCount => "🔢",
            LeaderboardFunction::GetPlayer { .. } => "👤",
        }
    }
}

/// Leaderboard entry as returned by the contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractLeaderboardEntry {
    pub address: String,
    pub wins: u32,
}

/// Player information from the contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractPlayer {
    pub address: String,
    pub wins: u32,
    pub joined_at: u64,
}

/// XDR generation request for contract functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractXdrRequest {
    pub source_account: String,
    pub function: LeaderboardFunction,
    pub wallet_type: Option<String>,
}

impl ContractXdrRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.source_account.is_empty() {
            return Err("Source account cannot be empty".to_string());
        }

        if !self.source_account.starts_with('G') || self.source_account.len() != 56 {
            return Err("Invalid source account format".to_string());
        }

        Ok(())
    }

    pub fn get_function(&self) -> &LeaderboardFunction {
        &self.function
    }
}

/// XDR generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractXdrResponse {
    pub success: bool,
    pub xdr: Option<String>,
    pub message: String,
}

impl ContractXdrResponse {
    pub fn success(xdr: String, message: String) -> Self {
        Self {
            success: true,
            xdr: Some(xdr),
            message,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            xdr: None,
            message,
        }
    }
}

/// Transaction submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSubmitRequest {
    pub signed_xdr: String,
    pub function: LeaderboardFunction,
    pub wallet_type: Option<String>,
}

impl ContractSubmitRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.signed_xdr.is_empty() {
            return Err("Signed XDR cannot be empty".to_string());
        }

        if self.signed_xdr.len() < 100 {
            return Err("Signed XDR appears too short to be valid".to_string());
        }

        Ok(())
    }

    pub fn get_function(&self) -> &LeaderboardFunction {
        &self.function
    }
}

/// Transaction submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSubmitResponse {
    pub success: bool,
    pub transaction_hash: Option<String>,
    pub result: Option<String>,
    pub message: String,
}

impl ContractSubmitResponse {
    pub fn success(result: String, hash: String, message: String) -> Self {
        Self {
            success: true,
            transaction_hash: Some(hash),
            result: Some(result),
            message,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            transaction_hash: None,
            result: None,
            message,
        }
    }
}

/// Leaderboard query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<u32>,
}

impl Default for LeaderboardQuery {
    fn default() -> Self {
        Self {
            limit: Some(10),
        }
    }
}

/// Combined leaderboard response that includes both contract data and additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntryWithMetadata>,
    pub total_players: u32,
    pub last_updated: Option<String>,
}

/// Enhanced leaderboard entry with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntryWithMetadata {
    pub rank: u32,
    pub address: String,
    pub wins: u32,
    pub username: Option<String>, // From game database if available
    pub joined_at: Option<u64>,
}