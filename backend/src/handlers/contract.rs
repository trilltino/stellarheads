use axum::{extract::Query, response::Json, extract::State};
use sqlx::PgPool;
use tracing::{info, warn};
use shared::dto::contract::{
    ContractXdrRequest, ContractXdrResponse, ContractSubmitRequest, ContractSubmitResponse,
    LeaderboardQuery, LeaderboardResponse, LeaderboardEntryWithMetadata, LeaderboardFunction
};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, Result},
    services::{soroban::client::{generate_leaderboard_xdr, submit_signed_transaction, ContractConfig}, GameService},
};

#[derive(Debug, Deserialize)]
pub struct JoinStatusQuery {
    pub player_address: String,
}

#[derive(Debug, Serialize)]
pub struct JoinStatusResponse {
    pub player_address: String,
    pub has_joined: bool,
    pub needs_join_xdr: bool,
}

pub async fn generate_contract_xdr_handler(
    State(_pool): State<PgPool>,
    Json(request): Json<ContractXdrRequest>,
) -> Result<Json<ContractXdrResponse>> {
    let wallet_info = request.wallet_type.as_deref().unwrap_or("unknown");
    info!("Contract XDR generation request received for account: {} (wallet: {})",
          request.source_account, wallet_info);

    request.validate().map_err(AppError::InvalidInput)?;

    let function = request.get_function().clone();
    let source_account = request.source_account.clone();

    info!("Selected function: {} ({})", function.name(), function.signature());

    let config = ContractConfig::default();

    info!("Generating XDR for account: {}...{}, function: {}",
          &source_account[..6], &source_account[source_account.len()-6..], function.name());

    let result = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            generate_leaderboard_xdr(&config, &source_account, &function).await
        })
    }).await;

    match result {
        Ok(Ok(xdr)) => {
            info!("✅ XDR generated successfully for Freighter wallet signing");
            Ok(Json(ContractXdrResponse::success(
                xdr,
                "XDR generated successfully for Freighter wallet".to_string(),
            )))
        }
        Ok(Err(error)) => {
            warn!("❌ XDR generation failed: {}", error);
            Err(AppError::XdrEncoding(error.to_string()))
        }
        Err(join_error) => {
            warn!("❌ Task join failed: {}", join_error);
            Err(AppError::TaskExecution(join_error.to_string()))
        }
    }
}

pub async fn submit_contract_transaction_handler(
    State(_pool): State<PgPool>,
    Json(request): Json<ContractSubmitRequest>,
) -> Result<Json<ContractSubmitResponse>> {
    let wallet_info = request.wallet_type.as_deref().unwrap_or("unknown");
    info!("Contract transaction submission request received from {} wallet", wallet_info);
    info!("Signed XDR length: {} characters", request.signed_xdr.len());

    request.validate().map_err(AppError::InvalidInput)?;

    let function = request.get_function().clone();
    let signed_xdr = request.signed_xdr.clone();

    let result = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            submit_signed_transaction(&signed_xdr, &function).await
        })
    }).await;

    match result {
        Ok(Ok((hash, contract_result))) => {
            info!("✅ Transaction submitted successfully: {}", hash);
            Ok(Json(ContractSubmitResponse::success(
                contract_result,
                hash,
                "Contract executed successfully".to_string(),
            )))
        }
        Ok(Err(error)) => {
            warn!("❌ Transaction submission failed: {}", error);
            Err(AppError::Transaction(error.to_string()))
        }
        Err(join_error) => {
            warn!("❌ Task join failed: {}", join_error);
            Err(AppError::TaskExecution(join_error.to_string()))
        }
    }
}

pub async fn get_leaderboard_handler(
    State(pool): State<PgPool>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardResponse>> {
    info!("Leaderboard request received with limit: {:?}", query.limit);

    let limit = query.limit.unwrap_or(10);

    // Get leaderboard data from database (traditional game results)
    let leaderboard_result = GameService::get_leaderboard(&pool, Some(limit as i64))
        .await?;

    // Convert to the contract-compatible format
    let entries_with_metadata: Vec<LeaderboardEntryWithMetadata> = leaderboard_result
        .items
        .into_iter()
        .enumerate()
        .map(|(index, entry)| LeaderboardEntryWithMetadata {
            rank: (index + 1) as u32,
            address: entry.wallet_address,
            wins: entry.wins as u32,
            username: Some(entry.username),
            joined_at: None, // Could be populated from contract if needed
        })
        .collect();

    let total_players = GameService::get_total_players(&pool)
        .await?;

    let response = LeaderboardResponse {
        entries: entries_with_metadata,
        total_players: total_players as u32,
        last_updated: Some(chrono::Utc::now().to_rfc3339()),
    };

    info!("✅ Leaderboard response ready with {} entries", response.entries.len());
    Ok(Json(response))
}

pub async fn check_join_status_handler(
    State(_pool): State<PgPool>,
    Query(query): Query<JoinStatusQuery>,
) -> Result<Json<JoinStatusResponse>> {
    info!("Join status check request for player: {}", query.player_address);

    let config = ContractConfig::default();
    let has_joined_function = LeaderboardFunction::HasJoined {
        player: query.player_address.clone()
    };

    // Clone variables for the spawned task
    let player_address_clone = query.player_address.clone();

    // Generate XDR for has_joined query (this would be used to call the contract)
    let xdr_result = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            generate_leaderboard_xdr(&config, &player_address_clone, &has_joined_function).await
        })
    }).await;

    match xdr_result {
        Ok(Ok(_xdr)) => {
            info!("✅ Join status XDR generated successfully");
            // For simplicity, we'll assume new players need to join
            // In production, you'd execute the XDR against the contract to get the actual status
            let response = JoinStatusResponse {
                player_address: query.player_address.clone(),
                has_joined: false, // Simplified - assume they need to join
                needs_join_xdr: true,
            };
            info!("📊 Join status result: player {} needs to join", query.player_address);
            Ok(Json(response))
        },
        Ok(Err(e)) => {
            warn!("❌ Failed to generate join status XDR: {}", e);
            Err(AppError::XdrEncoding(e.to_string()))
        },
        Err(join_error) => {
            warn!("❌ Join status task failed: {}", join_error);
            Err(AppError::TaskExecution(join_error.to_string()))
        }
    }
}

pub async fn contract_health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "stellar-heads-contract",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "contract_features": [
            "XDR generation",
            "Transaction submission",
            "Leaderboard integration",
            "Join status checking"
        ]
    }))
}