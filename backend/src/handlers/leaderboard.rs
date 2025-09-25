use axum::{
    extract::{Json, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::soroban::cli_client::SorobanCliClient;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerStats {
    pub wins: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LeaderboardEntry {
    pub player_address: String,
    pub wins: u32,
}

#[derive(Debug, Deserialize)]
pub struct JoinLeaderboardRequest {
    pub player_address: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct JoinLeaderboardResponse {
    pub transaction_xdr: String,
    pub network_passphrase: String,
    pub message: String,
    pub already_joined: bool,
}

#[derive(Debug, Deserialize)]
pub struct RecordGameResultRequest {
    pub player_address: String,
    pub username: String,
    pub won: bool,
}

#[derive(Debug, Serialize)]
pub struct RecordGameResultResponse {
    pub transaction_xdr: String,
    pub network_passphrase: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitSignedTransactionRequest {
    pub signed_xdr: String,
    pub transaction_hash: String,
    pub player_address: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionResult {
    pub hash: String,
    pub success: bool,
    pub ledger: u32,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerStatsQuery {
    pub player_address: String,
}

#[derive(Debug, Serialize)]
pub struct PlayerStatsResponse {
    pub player_address: String,
    pub stats: Option<PlayerStats>,
    pub has_joined: bool,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub total_players: usize,
}

fn get_contract_address() -> String {
    std::env::var("LEADERBOARD_CONTRACT_ADDRESS")
        .unwrap_or_else(|_| "CDQCZWN3W4DRVGIT23RWZ5GSU5XYDNGGRCOSKQG6ZLD5DWBMEAEKZG6N".to_string())
}

fn create_cli_client() -> SorobanCliClient {
    let contract_address = get_contract_address();
    let use_testnet = true;

    SorobanCliClient::new(contract_address, use_testnet)
}

pub async fn join_leaderboard(
    Json(req): Json<JoinLeaderboardRequest>,
) -> Result<(StatusCode, Json<JoinLeaderboardResponse>), (StatusCode, Json<serde_json::Value>)> {

    let client = create_cli_client();

    let already_joined = match client.has_joined(&req.player_address).await {
        Ok(joined) => joined,
        Err(_) => false,
    };

    if already_joined {
        let response = JoinLeaderboardResponse {
            transaction_xdr: String::new(),
            network_passphrase: String::new(),
            message: format!("User {} has already joined the leaderboard", req.username),
            already_joined: true,
        };
        return Ok((StatusCode::OK, Json(response)));
    }

    match client.create_join_transaction(&req.player_address).await {
        Ok(transaction_request) => {
            let response = JoinLeaderboardResponse {
                transaction_xdr: transaction_request.transaction_xdr,
                network_passphrase: transaction_request.network_passphrase,
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

pub async fn record_game_result(
    Json(req): Json<RecordGameResultRequest>,
) -> Result<(StatusCode, Json<RecordGameResultResponse>), (StatusCode, Json<serde_json::Value>)> {

    let client = create_cli_client();

    let has_joined = match client.has_joined(&req.player_address).await {
        Ok(joined) => joined,
        Err(_) => false,
    };

    if !has_joined {
        let error_response = serde_json::json!({
            "error": "Player must join leaderboard first",
            "details": format!("Player {} has not joined the leaderboard", req.username)
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let transaction_request = if req.won {
        client.create_add_win_transaction(&req.player_address).await
    } else {
        let response = RecordGameResultResponse {
            transaction_xdr: String::new(),
            network_passphrase: String::new(),
            message: format!("Loss recorded locally for {} (simple contract only tracks wins)", req.username),
        };
        return Ok((StatusCode::OK, Json(response)));
    };

    match transaction_request {
        Ok(tx_req) => {
            let result_type = if req.won { "win" } else { "loss" };
            let response = RecordGameResultResponse {
                transaction_xdr: tx_req.transaction_xdr,
                network_passphrase: tx_req.network_passphrase,
                message: format!("Record {} transaction ready for {}. Please sign with Freighter.", result_type, req.username),
            };
            Ok((StatusCode::OK, Json(response)))
        },
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create game result transaction",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn submit_signed_transaction(
    Json(req): Json<SubmitSignedTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResult>), (StatusCode, Json<serde_json::Value>)> {

    let client = create_cli_client();

    match client.submit_transaction(&req.signed_xdr).await {
        Ok(ledger) => {
            let transaction_result = TransactionResult {
                hash: req.transaction_hash,
                success: true,
                ledger,
                error: None,
            };
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

pub async fn get_player_stats(
    Query(params): Query<PlayerStatsQuery>,
) -> Result<(StatusCode, Json<PlayerStatsResponse>), (StatusCode, Json<serde_json::Value>)> {

    let client = create_cli_client();

    let has_joined = match client.has_joined(&params.player_address).await {
        Ok(joined) => joined,
        Err(_) => false,
    };

    if !has_joined {
        let response = PlayerStatsResponse {
            player_address: params.player_address,
            stats: None,
            has_joined: false,
        };
        return Ok((StatusCode::OK, Json(response)));
    }

    let wins = match client.get_wins(&params.player_address).await {
        Ok(wins) => wins,
        Err(_) => 0,
    };

    let stats = Some(PlayerStats { wins });

    let response = PlayerStatsResponse {
        player_address: params.player_address,
        stats,
        has_joined: true,
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_leaderboard(
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {

    let _client = create_cli_client();

    let response = serde_json::json!({
        "entries": [],
        "total_players": 0,
        "message": "Simple contract doesn't support leaderboard listing. Use individual player queries."
    });

    Ok((StatusCode::OK, Json(response)))
}

pub async fn check_player_joined(
    Query(params): Query<PlayerStatsQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {

    let client = create_cli_client();

    match client.has_joined(&params.player_address).await {
        Ok(has_joined) => {
            let response = serde_json::json!({
                "player_address": params.player_address,
                "has_joined": has_joined,
                "message": if has_joined {
                    "Player has joined the leaderboard"
                } else {
                    "Player has not joined the leaderboard"
                }
            });
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

pub async fn test_add_win(
    Query(params): Query<PlayerStatsQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {

    let client = create_cli_client();

    let has_joined = match client.has_joined(&params.player_address).await {
        Ok(joined) => joined,
        Err(_) => false,
    };

    if !has_joined {
        let error_response = serde_json::json!({
            "error": "Player must join leaderboard first",
            "player_address": params.player_address
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let current_wins = match client.get_wins(&params.player_address).await {
        Ok(wins) => wins,
        Err(_) => 0,
    };

    match client.create_add_win_transaction(&params.player_address).await {
        Ok(tx_req) => {
            let response = serde_json::json!({
                "success": true,
                "message": format!("Add win transaction ready for {}", params.player_address),
                "current_wins": current_wins,
                "transaction_xdr": tx_req.transaction_xdr,
                "network_passphrase": tx_req.network_passphrase,
                "requires_signature": true
            });
            Ok((StatusCode::OK, Json(response)))
        },
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Failed to create add win transaction",
                "details": e.to_string()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}