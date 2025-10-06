use serde::{Deserialize, Serialize};
use std::fmt;

/// Newtype wrapper for wallet addresses providing type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WalletAddress(String);

impl WalletAddress {
    /// Create a new WalletAddress with validation
    pub fn new(address: impl Into<String>) -> Result<Self, String> {
        let address = address.into();

        // Basic Stellar address validation (G... for public key, starts with G, 56 chars)
        if address.starts_with('G') && address.len() == 56 {
            Ok(Self(address))
        } else {
            Err(format!("Invalid wallet address format: {address}"))
        }
    }

    /// Create without validation (use with caution, for trusted sources)
    pub fn new_unchecked(address: impl Into<String>) -> Self {
        Self(address.into())
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to String
    pub fn into_string(self) -> String {
        self.0
    }

    /// Get a truncated display version (first 4 and last 4 chars)
    pub fn truncated(&self) -> String {
        if self.0.len() > 8 {
            format!("{}...{}", &self.0[..4], &self.0[self.0.len()-4..])
        } else {
            self.0.clone()
        }
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for WalletAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Newtype wrapper for game modes
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    #[default]
    SinglePlayerVsAi,
    Multiplayer,
    Practice,
}

impl GameMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameMode::SinglePlayerVsAi => "single_player_vs_ai",
            GameMode::Multiplayer => "multiplayer",
            GameMode::Practice => "practice",
        }
    }

    /// Parse from string with fallback to default
    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "single_player_vs_ai" | "single_player" => GameMode::SinglePlayerVsAi,
            "multiplayer" => GameMode::Multiplayer,
            "practice" => GameMode::Practice,
            _ => GameMode::SinglePlayerVsAi, // Default fallback
        }
    }
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Newtype wrapper for game session IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameSessionId(String);

impl GameSessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for GameSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for GameSessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Newtype wrapper for usernames
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Username(String);

impl Username {
    /// Create a new Username with validation
    pub fn new(name: impl Into<String>) -> Result<Self, String> {
        let name = name.into();

        // Basic validation: not empty, reasonable length
        if name.is_empty() {
            Err("Username cannot be empty".to_string())
        } else if name.len() > 50 {
            Err("Username too long (max 50 characters)".to_string())
        } else {
            Ok(Self(name))
        }
    }

    /// Create without validation
    pub fn new_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_address_validation() {
        // Valid address
        let valid = "GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ";
        assert!(WalletAddress::new(valid).is_ok());

        // Invalid: wrong length
        assert!(WalletAddress::new("GCRBGOBUEZCZF5").is_err());

        // Invalid: doesn't start with G
        assert!(WalletAddress::new("ACRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ").is_err());
    }

    #[test]
    fn test_wallet_address_truncated() {
        let addr = WalletAddress::new_unchecked("GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ");
        assert_eq!(addr.truncated(), "GCRB...JMBZ");
    }

    #[test]
    fn test_game_mode_parsing() {
        assert_eq!(GameMode::from_str_or_default("single_player"), GameMode::SinglePlayerVsAi);
        assert_eq!(GameMode::from_str_or_default("multiplayer"), GameMode::Multiplayer);
        assert_eq!(GameMode::from_str_or_default("invalid"), GameMode::SinglePlayerVsAi);
    }

    #[test]
    fn test_username_validation() {
        assert!(Username::new("ValidUser").is_ok());
        assert!(Username::new("").is_err());
        assert!(Username::new("a".repeat(51)).is_err());
    }
}
