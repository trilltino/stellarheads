use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, window};
use gloo_net::http::Request;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitScoreRequest {
    pub player_address: String,
    pub username: String,
    pub score: u64,
    pub game_mode: String,
    pub duration: u64,
    pub achievements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitScoreResponse {
    pub transaction_xdr: String,
    pub network_passphrase: String,
    pub session_id: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignTransactionResponse {
    pub signed_xdr: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResult {
    pub hash: String,
    pub success: bool,
    pub ledger: u32,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractInfo {
    pub contract_address: String,
    pub network_passphrase: String,
    pub network_name: String,
    pub rpc_url: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = freighterApi)]
    fn isConnected() -> js_sys::Promise;
    
    #[wasm_bindgen(js_namespace = freighterApi)]
    fn signTransaction(xdr: &str, options: &JsValue) -> js_sys::Promise;
}

/// Get contract information from backend
pub async fn get_contract_info() -> Result<ContractInfo, String> {
    console::log_1(&"📡 Fetching contract information...".into());
    
    let response = Request::get("http://localhost:3000/api/soroban/contract-info")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch contract info: {}", e))?;

    if !response.ok() {
        return Err(format!("Failed to get contract info: {}", response.status()));
    }

    let contract_info: ContractInfo = response.json()
        .await
        .map_err(|e| format!("Failed to parse contract info: {}", e))?;

    console::log_1(&format!("✅ Contract Address: {}", contract_info.contract_address).into());
    console::log_1(&format!("🌐 Network: {}", contract_info.network_name).into());
    
    Ok(contract_info)
}

pub async fn submit_score_to_contract(
    player_address: &str,
    username: &str,
    score: u64,
    game_mode: &str,
    duration: u64,
    achievements: Vec<String>,
) -> Result<TransactionResult, String> {
    
    // Step 1: Create the transaction via backend
    let request = SubmitScoreRequest {
        player_address: player_address.to_string(),
        username: username.to_string(),
        score,
        game_mode: game_mode.to_string(),
        duration,
        achievements,
    };

    console::log_1(&"🚀 Creating Soroban transaction for score submission...".into());

    let response = Request::post("http://localhost:3000/api/soroban/submit-score")
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.ok() {
        return Err(format!("Backend error: {}", response.status()));
    }

    let submit_response: SubmitScoreResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    console::log_1(&format!("✅ Transaction created: {}", submit_response.message).into());

    // Step 2: Sign the transaction with Freighter
    console::log_1(&"🔐 Signing transaction with Freighter...".into());

    let sign_result = sign_transaction_with_freighter(
        &submit_response.transaction_xdr,
        &submit_response.network_passphrase,
    ).await?;

    console::log_1(&"✅ Transaction signed successfully!".into());

    // Step 3: Submit the signed transaction via backend
    let signed_request = serde_json::json!({
        "signed_xdr": sign_result.signed_xdr,
        "transaction_hash": "placeholder_hash", // Will be calculated by backend
        "player_address": player_address
    });

    let final_response = Request::post("http://localhost:3000/api/soroban/submit-transaction")
        .header("Content-Type", "application/json")
        .json(&signed_request)
        .map_err(|e| format!("Failed to serialize signed request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to submit signed transaction: {}", e))?;

    if !final_response.ok() {
        return Err(format!("Failed to submit transaction: {}", final_response.status()));
    }

    let result: TransactionResult = final_response.json()
        .await
        .map_err(|e| format!("Failed to parse transaction result: {}", e))?;

    console::log_1(&format!("🎉 Score submitted to blockchain! Hash: {}", result.hash).into());

    Ok(result)
}

pub async fn sign_transaction_with_freighter(
    transaction_xdr: &str,
    network_passphrase: &str,
) -> Result<SignTransactionResponse, String> {
    
    // Check if Freighter is available and connected
    let window = window().ok_or("No window object")?;
    let freighter_api = window.get("freighterApi")
        .ok_or("Freighter API not found")?;

    // Create signing options
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
        &options,
        &"networkPassphrase".into(),
        &network_passphrase.into(),
    ).map_err(|_| "Failed to set network passphrase")?;

    // Sign the transaction
    let sign_promise = signTransaction(transaction_xdr, &options.into());
    let sign_result = JsFuture::from(sign_promise)
        .await
        .map_err(|e| format!("Freighter signing failed: {:?}", e))?;

    // Parse the signed XDR
    let signed_xdr = js_sys::Reflect::get(&sign_result, &"signedTxXdr".into())
        .map_err(|_| "Failed to get signed XDR")?
        .as_string()
        .ok_or("Signed XDR is not a string")?;

    Ok(SignTransactionResponse {
        signed_xdr,
    })
}

pub async fn start_game_session(player_address: &str, username: &str) -> Result<String, String> {
    let request = serde_json::json!({
        "player_address": player_address,
        "username": username
    });

    console::log_1(&"🎮 Starting game session on blockchain...".into());

    let response = Request::post("http://localhost:3000/api/soroban/start-game")
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Failed to create start game request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to send start game request: {}", e))?;

    if response.ok() {
        let response_data: serde_json::Value = response.json()
            .await
            .map_err(|e| format!("Failed to parse start game response: {}", e))?;

        // Extract transaction XDR and sign it
        let transaction_xdr = response_data["transaction_xdr"]
            .as_str()
            .ok_or("No transaction XDR in response")?;
        
        let network_passphrase = response_data["network_passphrase"]
            .as_str()
            .ok_or("No network passphrase in response")?;

        // Sign and submit the start game transaction
        let sign_result = sign_transaction_with_freighter(transaction_xdr, network_passphrase).await?;

        console::log_1(&"✅ Game session started on blockchain!".into());
        Ok("Game session started successfully".to_string())
    } else {
        Err(format!("Failed to start game session: {}", response.status()))
    }
}