use axum::{
    extract::{Json, State, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::database::connection::DbPool;
use crate::soroban::{
    client::SorobanJoinClient,
    types::{TransactionResult}
};


#[derive(Debug, Deserialize)]
pub struct JoinRequest {
    pub player_address: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct JoinResponse {
    pub transaction_xdr: String,
    pub network_passphrase: String,
    pub message: String,
    pub already_joined: bool,
}

#[derive(Debug, Deserialize)]
pub struct StartGameRequest {
    pub player_address: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitSignedTransactionRequest {
    pub signed_xdr: String,
    pub transaction_hash: String,
    pub player_address: String,
}

/// Create a transaction for joining the contract
pub async fn create_join_transaction(
    State(_pool): State<DbPool>,
    Json(req): Json<JoinRequest>,
) -> Result<(StatusCode, Json<JoinResponse>), (StatusCode, Json<serde_json::Value>)> {
    
    // TODO: Get contract address from environment or database
    let contract_address = std::env::var("JOIN_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CDDG3FABIMQ2STFKNXJXDYOBU6U37G2JSD4DSF4AM4YHAEIYCC4WDNCI".to_string());

    // Use testnet for development
    let client = match SorobanJoinClient::new(contract_address, true) {
        Ok(c) => c,
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create Soroban client",
                "details": e.to_string()
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    // First check if already joined
    let already_joined = match client.has_joined(&req.player_address).await {
        Ok(joined) => joined,
        Err(_) => false, // If error checking, assume not joined
    };

    if already_joined {
        let response = JoinResponse {
            transaction_xdr: String::new(),
            network_passphrase: String::new(),
            message: format!("User {} has already joined the contract", req.username),
            already_joined: true,
        };
        return Ok((StatusCode::OK, Json(response)));
    }

    match client.create_join_transaction(&req.player_address).await {
        Ok(transaction_response) => {
            let response = JoinResponse {
                transaction_xdr: transaction_response.transaction_xdr,
                network_passphrase: transaction_response.network_passphrase,
                message: format!("Join transaction ready for {}. Please sign with Freighter.", req.username),
                already_joined: false,
            };

            Ok((StatusCode::OK, Json(response)))
        },
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create join transaction",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Check if a player has already joined the contract
pub async fn check_player_joined(
    State(_pool): State<DbPool>,
    Query(params): Query<PlayerJoinedQuery>,
) -> Result<(StatusCode, Json<PlayerJoinedResponse>), (StatusCode, Json<serde_json::Value>)> {
    
    let contract_address = std::env::var("JOIN_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CDDG3FABIMQ2STFKNXJXDYOBU6U37G2JSD4DSF4AM4YHAEIYCC4WDNCI".to_string());

    let client = match SorobanJoinClient::new(contract_address, true) {
        Ok(c) => c,
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create Soroban client",
                "details": e.to_string()
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    match client.has_joined(&params.player_address).await {
        Ok(has_joined) => {
            let response = PlayerJoinedResponse {
                player_address: params.player_address,
                has_joined,
                message: if has_joined {
                    "Player has joined the contract".to_string()
                } else {
                    "Player has not joined the contract".to_string()
                },
            };
            Ok((StatusCode::OK, Json(response)))
        },
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to check join status",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Submit a signed transaction to the network
pub async fn submit_signed_transaction(
    State(_pool): State<DbPool>,
    Json(req): Json<SubmitSignedTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResult>), (StatusCode, Json<serde_json::Value>)> {
    
    let contract_address = std::env::var("JOIN_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CDDG3FABIMQ2STFKNXJXDYOBU6U37G2JSD4DSF4AM4YHAEIYCC4WDNCI".to_string());

    let client = match SorobanJoinClient::new(contract_address, true) {
        Ok(c) => c,
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create Soroban client",
                "details": e.to_string()
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };
    
    // Submit the signed transaction
    match client.submit_transaction(&req.signed_xdr).await {
        Ok(result) => {
            let transaction_result = TransactionResult {
                hash: req.transaction_hash,
                success: true,
                ledger: result.ledger,
                error: None,
            };

            // TODO: Update database with transaction result if needed
            // You might want to store successful joins, update user status, etc.

            Ok((StatusCode::OK, Json(transaction_result)))
        },
        Err(e) => {
            let transaction_result = TransactionResult {
                hash: req.transaction_hash,
                success: false,
                ledger: 0,
                error: Some(e.to_string()),
            };

            Ok((StatusCode::OK, Json(transaction_result)))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayerJoinedQuery {
    pub player_address: String,
}

#[derive(Debug, Serialize)]
pub struct PlayerJoinedResponse {
    pub player_address: String,
    pub has_joined: bool,
    pub message: String,
}

/// Get all joined players from the contract
pub async fn get_joined_players(
    State(_pool): State<DbPool>,
) -> Result<(StatusCode, Json<JoinedPlayersResponse>), (StatusCode, Json<serde_json::Value>)> {
    
    let contract_address = std::env::var("JOIN_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CDDG3FABIMQ2STFKNXJXDYOBU6U37G2JSD4DSF4AM4YHAEIYCC4WDNCI".to_string());

    let client = match SorobanJoinClient::new(contract_address, true) {
        Ok(c) => c,
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create Soroban client",
                "details": e.to_string()
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    match client.get_joined().await {
        Ok(joined_addresses) => {
            let count = client.get_count().await.unwrap_or(0);
            
            let response = JoinedPlayersResponse {
                joined_addresses,
                count,
                message: format!("Found {} joined players", count),
            };
            Ok((StatusCode::OK, Json(response)))
        },
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to get joined players",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JoinedPlayersResponse {
    pub joined_addresses: Vec<String>,
    pub count: u32,
    pub message: String,
}

