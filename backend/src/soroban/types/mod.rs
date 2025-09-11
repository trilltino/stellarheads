use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameScore {
    pub player_address: String,
    pub username: String,
    pub score: u64,
    pub game_mode: String,
    pub duration: u64, // in seconds
    pub achievements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardRequest {
    pub player_address: String,
    pub score: GameScore,
    pub signature_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallRequest {
    pub contract_address: String,
    pub function_name: String,
    pub parameters: serde_json::Value,
    pub player_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallResponse {
    pub transaction_xdr: String,
    pub network_passphrase: String,
    pub success: bool,
    pub message: String,
    pub estimated_fee: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub signed_xdr: String,
    pub transaction_hash: String,
    pub player_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub hash: String,
    pub success: bool,
    pub ledger: u32,
    pub error: Option<String>,
}