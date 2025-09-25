use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guest {
    pub username: String,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserType {
    Guest,
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserType::Guest => write!(f, "Guest"),
        }
    }
}
