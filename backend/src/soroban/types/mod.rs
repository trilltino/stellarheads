use serde::{Deserialize, Serialize};

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