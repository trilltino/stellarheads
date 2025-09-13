use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, window};
use gloo_net::http::Request;

// ===== BACKEND-MATCHING TYPES =====

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequest {
    pub player_address: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinResponse {
    pub transaction_xdr: String,
    pub network_passphrase: String,
    pub message: String,
    pub already_joined: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerJoinedResponse {
    pub player_address: String,
    pub has_joined: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinedPlayersResponse {
    pub joined_addresses: Vec<String>,
    pub count: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub contract_address: String,
    pub network_passphrase: String,
    pub network_name: String,
    pub rpc_url: String,
}

// ===== FREIGHTER BINDINGS =====

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = freighterApi)]
    fn isConnected() -> js_sys::Promise;
    
    #[wasm_bindgen(js_namespace = freighterApi)]
    fn signTransaction(xdr: &str, options: &JsValue) -> js_sys::Promise;
}

// ===== MAIN JOIN FUNCTIONALITY =====

/// Join the contract - creates transaction for user to sign
pub async fn join_contract(
    player_address: &str,
    username: &str,
) -> Result<JoinResponse, String> {
    
    let request = JoinRequest {
        player_address: player_address.to_string(),
        username: username.to_string(),
    };

    console::log_1(&format!("🚀 Creating join transaction for user: {}", username).into());

    let response = Request::post("http://localhost:3000/join")
        .header("Content-Type", "application/json")
        .json(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.ok() {
        return Err(format!("Backend error: {}", response.status()));
    }

    let join_response: JoinResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    console::log_1(&format!("✅ Join transaction created: {}", join_response.message).into());

    Ok(join_response)
}

/// Complete join flow: create transaction + sign with Freighter + submit
pub async fn complete_join_flow(
    player_address: &str,
    username: &str,
) -> Result<TransactionResult, String> {
    
    // Step 1: Create join transaction
    let join_response = join_contract(player_address, username).await?;

    // Check if already joined
    if join_response.already_joined {
        console::log_1(&"ℹ️ User already joined, skipping transaction".into());
        return Err("User has already joined the contract".to_string());
    }

    // Step 2: Sign transaction with Freighter
    console::log_1(&"🔐 Signing transaction with Freighter...".into());
    
    let sign_result = sign_transaction_with_freighter(
        &join_response.transaction_xdr,
        &join_response.network_passphrase,
    ).await?;

    console::log_1(&"✅ Transaction signed successfully!".into());

    // Step 3: Submit signed transaction
    let submit_request = serde_json::json!({
        "signed_xdr": sign_result.signed_xdr,
        "transaction_hash": "placeholder_hash", // Backend will calculate
        "player_address": player_address
    });

    let response = Request::post("http://localhost:3000/submit-signed-transaction")
        .header("Content-Type", "application/json")
        .json(&submit_request)
        .map_err(|e| format!("Failed to serialize submit request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to submit transaction: {}", e))?;

    if !response.ok() {
        return Err(format!("Failed to submit transaction: {}", response.status()));
    }

    let result: TransactionResult = response.json()
        .await
        .map_err(|e| format!("Failed to parse result: {}", e))?;

    console::log_1(&format!("🎉 Successfully joined contract! Hash: {}", result.hash).into());

    Ok(result)
}

/// Check if a player has already joined
pub async fn check_player_joined(player_address: &str) -> Result<PlayerJoinedResponse, String> {
    console::log_1(&format!("📡 Checking if {} has joined...", player_address).into());
    let response = Request::get(&format!("http://localhost:3000/check-joined?player_address={}", player_address))
        .send()
        .await
        .map_err(|e| format!("Failed to check join status: {}", e))?;

    if !response.ok() {
        return Err(format!("Backend error: {}", response.status()));
    }
    let status: PlayerJoinedResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    console::log_1(&format!("✅ Join status: {}", status.message).into());

    Ok(status)
}

/// Get all players who have joined
pub async fn get_joined_players() -> Result<JoinedPlayersResponse, String> {
    console::log_1(&"📡 Fetching all joined players...".into());

    let response = Request::get("http://localhost:3000/joined-players")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch joined players: {}", e))?;

    if !response.ok() {
        return Err(format!("Backend error: {}", response.status()));
    }

    let players: JoinedPlayersResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    console::log_1(&format!("✅ Found {} joined players", players.count).into());

    Ok(players)
}

/// Sign transaction with Freighter wallet
pub async fn sign_transaction_with_freighter(
    transaction_xdr: &str,
    network_passphrase: &str,
) -> Result<SignTransactionResponse, String> {
    
    // Check if Freighter is available
    let window = window().ok_or("No window object")?;
    let _freighter_api = window.get("freighterApi")
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

/// Get mock contract info (since we don't have a real contract endpoint)
pub async fn get_contract_info() -> Result<ContractInfo, String> {
    console::log_1(&"📡 Getting contract information...".into());
    
    // Mock contract info since backend doesn't have this endpoint
    let contract_info = ContractInfo {
        contract_address: "CDDG3FABIMQ2STFKNXJXDYOBU6U37G2JSD4DSF4AM4YHAEIYCC4WDNCI".to_string(),
        network_passphrase: "Test SDF Network ; September 2015".to_string(),
        network_name: "Testnet".to_string(),
        rpc_url: "https://soroban-testnet.stellar.org".to_string(),
    };

    console::log_1(&format!("✅ Contract Address: {}", contract_info.contract_address).into());
    console::log_1(&format!("🌐 Network: {}", contract_info.network_name).into());
    
    Ok(contract_info)
}