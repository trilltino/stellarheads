use axum::{
    extract::{Json, State, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::database::connection::DbPool;
use crate::soroban::{
    client::SorobanLeaderboardClient,
    types::{ContractCallRequest, ContractCallResponse, GameScore, SignedTransaction, TransactionResult}
};
//use soroban_client::Network;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ContractInfo {
    pub contract_address: String,
    pub network_passphrase: String,
    pub network_name: String,
    pub rpc_url: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitScoreRequest {
    pub player_address: String,
    pub username: String,
    pub score: u64,
    pub game_mode: String,
    pub duration: u64,
    pub achievements: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitScoreResponse {
    pub transaction_xdr: String,
    pub network_passphrase: String,
    pub session_id: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct StartGameRequest {
    pub player_address: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct TestContractSignRequest {
    pub player_address: String,
    pub username: String,
    pub action: String, // "join_leaderboard", "submit_score", etc.
}

#[derive(Debug, Deserialize)]
pub struct SubmitSignedTransactionRequest {
    pub signed_xdr: String,
    pub transaction_hash: String,
    pub player_address: String,
}

/// Create a transaction for submitting a score to the Soroban leaderboard contract
pub async fn create_submit_score_transaction(
    State(pool): State<DbPool>,
    Json(req): Json<SubmitScoreRequest>,
) -> Result<(StatusCode, Json<SubmitScoreResponse>), (StatusCode, Json<serde_json::Value>)> {
    
    // TODO: Get contract address from environment or database
    let contract_address = std::env::var("LEADERBOARD_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CBQHNAXSI55GX2GN6D67GK7BHVPSLJUGX5KPJXFYNO7PSRT4LANXICKF".to_string());

    // Use testnet for development
    let is_testnet = std::env::var("SOROBAN_NETWORK")
        .unwrap_or_else(|_| "testnet".to_string()) == "testnet";
    let client = SorobanLeaderboardClient::new(contract_address, is_testnet);

    match client.create_submit_score_transaction(
        &req.player_address,
        req.username.clone(),
        req.score,
    ).await {
        Ok(transaction_response) => {
            let session_id = Uuid::new_v4().to_string();
            
            // Store the game session in database for tracking
            // TODO: Add game_sessions table and store session data
            
            let response = SubmitScoreResponse {
                transaction_xdr: transaction_response.transaction_xdr,
                network_passphrase: transaction_response.network_passphrase,
                session_id,
                message: "Transaction ready for signing. Please sign with Freighter.".to_string(),
            };

            Ok((StatusCode::OK, Json(response)))
        },
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create transaction",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Create a transaction for starting a game session
pub async fn create_start_game_transaction(
    State(pool): State<DbPool>,
    Json(req): Json<StartGameRequest>,
) -> Result<(StatusCode, Json<ContractCallResponse>), (StatusCode, Json<serde_json::Value>)> {
    
    let contract_address = std::env::var("LEADERBOARD_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CBQHNAXSI55GX2GN6D67GK7BHVPSLJUGX5KPJXFYNO7PSRT4LANXICKF".to_string());

    let is_testnet = std::env::var("SOROBAN_NETWORK")
        .unwrap_or_else(|_| "testnet".to_string()) == "testnet";
    let client = SorobanLeaderboardClient::new(contract_address, is_testnet);
    let session_id = Uuid::new_v4().to_string();

    match client.create_start_session_transaction(
        &req.player_address,
        session_id.clone(),
    ).await {
        Ok(transaction_response) => {
            Ok((StatusCode::OK, Json(ContractCallResponse {
                transaction_xdr: transaction_response.transaction_xdr,
                network_passphrase: transaction_response.network_passphrase,
                success: true,
                message: format!("Game session {} ready to start", session_id),
                estimated_fee: Some(100_000), // 0.001 XLM estimated
            })))
        },
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create game start transaction",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Submit a signed transaction to the network
pub async fn submit_signed_transaction(
    State(pool): State<DbPool>,
    Json(req): Json<SubmitSignedTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResult>), (StatusCode, Json<serde_json::Value>)> {
    
    // TODO: Submit the signed transaction to the Stellar network
    // This would use the soroban-client to submit the transaction
    
    // For now, return a mock success response
    let result = TransactionResult {
        hash: req.transaction_hash,
        success: true,
        ledger: 12345, // Mock ledger number
        error: None,
    };

    // TODO: Update database with transaction result
    // Store the successful score submission, update leaderboards, etc.

    Ok((StatusCode::OK, Json(result)))
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<u32>,
    pub game_mode: Option<String>,
}

/// Get the current leaderboard (read-only, no transaction needed)
pub async fn get_leaderboard(
    State(pool): State<DbPool>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<(StatusCode, Json<Vec<GameScore>>), (StatusCode, Json<serde_json::Value>)> {
    
    let limit = params.limit.unwrap_or(10).min(100); // Max 100 entries
    
    // TODO: Query the Soroban contract for leaderboard data
    // For now, return mock data from database
    
    let mock_leaderboard = vec![
        GameScore {
            player_address: "GCAB...XYZ".to_string(),
            username: "TopPlayer".to_string(),
            score: 15000,
            game_mode: "classic".to_string(),
            duration: 300,
            achievements: vec!["high_score".to_string(), "speed_demon".to_string()],
        },
        GameScore {
            player_address: "GDEF...ABC".to_string(),
            username: "PlayerTwo".to_string(),
            score: 12500,
            game_mode: "classic".to_string(),
            duration: 280,
            achievements: vec!["steady_player".to_string()],
        },
    ];

    Ok((StatusCode::OK, Json(mock_leaderboard)))
}

#[derive(Debug, Deserialize)]
pub struct PlayerScoreQuery {
    pub player_address: String,
}

/// Get a specific player's best score
pub async fn get_player_score(
    State(pool): State<DbPool>,
    Query(params): Query<PlayerScoreQuery>,
) -> Result<(StatusCode, Json<Option<GameScore>>), (StatusCode, Json<serde_json::Value>)> {
    
    // TODO: Query the Soroban contract for player's best score
    // For now, return mock data
    
    if params.player_address == "GCAB...XYZ" {
        let player_score = GameScore {
            player_address: params.player_address,
            username: "Player".to_string(),
            score: 15000,
            game_mode: "classic".to_string(),
            duration: 300,
            achievements: vec!["high_score".to_string()],
        };
        Ok((StatusCode::OK, Json(Some(player_score))))
    } else {
        Ok((StatusCode::OK, Json(None)))
    }
}

/// Get contract information for frontend/Freighter integration
pub async fn get_contract_info() -> Result<(StatusCode, Json<ContractInfo>), (StatusCode, Json<serde_json::Value>)> {
    let contract_address = std::env::var("LEADERBOARD_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CBQHNAXSI55GX2GN6D67GK7BHVPSLJUGX5KPJXFYNO7PSRT4LANXICKF".to_string());
    
    let network_name = std::env::var("SOROBAN_NETWORK")
        .unwrap_or_else(|_| "testnet".to_string());
    
    let network_passphrase = std::env::var("NETWORK_PASSPHRASE")
        .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string());
    
    let rpc_url = std::env::var("SOROBAN_RPC_URL")
        .unwrap_or_else(|_| "https://soroban-testnet.stellar.org:443".to_string());

    let contract_info = ContractInfo {
        contract_address,
        network_passphrase,
        network_name,
        rpc_url,
    };

    Ok((StatusCode::OK, Json(contract_info)))
}

/// TEST: Create a leaderboard contract transaction for user signing
pub async fn test_contract_signing(
    State(pool): State<DbPool>,
    Json(req): Json<TestContractSignRequest>,
) -> Result<(StatusCode, Json<ContractCallResponse>), (StatusCode, Json<serde_json::Value>)> {
    
    println!("🧪 Testing contract signing for user: {}", req.username);
    println!("📋 Action: {}", req.action);
    println!("🔑 Player Address: {}", req.player_address);
    
    let contract_address = std::env::var("LEADERBOARD_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CBQHNAXSI55GX2GN6D67GK7BHVPSLJUGX5KPJXFYNO7PSRT4LANXICKF".to_string());

    let network = Network::Testnet;
    let client = SorobanLeaderboardClient::new(contract_address.clone(), network);

    match req.action.as_str() {
        "join_leaderboard" => {
            // Create a transaction to initialize player on leaderboard
            match client.create_submit_score_transaction(
                &req.player_address,
                req.username.clone(),
                0, // Initial score of 0 to "join" the leaderboard
            ).await {
                Ok(transaction_response) => {
                    println!("✅ Join leaderboard transaction created successfully");
                    
                    Ok((StatusCode::OK, Json(ContractCallResponse {
                        transaction_xdr: transaction_response.transaction_xdr,
                        network_passphrase: transaction_response.network_passphrase,
                        success: true,
                        message: format!("🌟 Ready to join leaderboard! Contract: {}...{}", 
                            &contract_address[0..8], 
                            &contract_address[contract_address.len()-8..]
                        ),
                        estimated_fee: Some(100_000), // 0.001 XLM
                    })))
                },
                Err(e) => {
                    let error_response = serde_json::json!({
                        "error": "Failed to create join leaderboard transaction",
                        "details": e.to_string(),
                        "contract_address": contract_address
                    });
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        },
        "test_score" => {
            // Create a test score submission (1000 points)
            match client.create_submit_score_transaction(
                &req.player_address,
                req.username.clone(),
                1000,
            ).await {
                Ok(transaction_response) => {
                    println!("✅ Test score transaction created successfully");
                    
                    Ok((StatusCode::OK, Json(ContractCallResponse {
                        transaction_xdr: transaction_response.transaction_xdr,
                        network_passphrase: transaction_response.network_passphrase,
                        success: true,
                        message: format!("🎯 Test score (1000 pts) ready for signing! Contract: {}...{}", 
                            &contract_address[0..8], 
                            &contract_address[contract_address.len()-8..]
                        ),
                        estimated_fee: Some(150_000), // 0.0015 XLM
                    })))
                },
                Err(e) => {
                    let error_response = serde_json::json!({
                        "error": "Failed to create test score transaction",
                        "details": e.to_string()
                    });
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        },
        "initialize_contract" => {
            // Initialize the contract (admin only)
            match client.create_initialize_transaction(&req.player_address).await {
                Ok(transaction_response) => {
                    println!("✅ Contract initialization transaction created");
                    
                    Ok((StatusCode::OK, Json(ContractCallResponse {
                        transaction_xdr: transaction_response.transaction_xdr,
                        network_passphrase: transaction_response.network_passphrase,
                        success: true,
                        message: format!("🏗️ Initialize contract ready for signing! You'll become admin of: {}...{}", 
                            &contract_address[0..8], 
                            &contract_address[contract_address.len()-8..]
                        ),
                        estimated_fee: Some(200_000), // 0.002 XLM
                    })))
                },
                Err(e) => {
                    let error_response = serde_json::json!({
                        "error": "Failed to create contract initialization transaction",
                        "details": e.to_string()
                    });
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        },
        _ => {
            let error_response = serde_json::json!({
                "error": "Unknown action",
                "available_actions": ["join_leaderboard", "test_score", "initialize_contract"],
                "provided_action": req.action
            });
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}