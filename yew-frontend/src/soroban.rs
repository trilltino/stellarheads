use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, window};

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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = freighterApi)]
    fn isConnected() -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = freighterApi)]
    fn signTransaction(xdr: &str, options: &JsValue) -> js_sys::Promise;
}

pub async fn post_join_contract(
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

    let join_response: JoinResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    console::log_1(&format!("✅ Join transaction created: {}", join_response.message).into());

    Ok(join_response)
}

pub async fn complete_join_flow(
    player_address: &str,
    username: &str,
) -> Result<TransactionResult, String> {
    let join_response = post_join_contract(player_address, username).await?;

    if join_response.already_joined {
        console::log_1(&"ℹ️ User already joined, skipping transaction".into());
        return Err("User has already joined the contract".to_string());
    }

    console::log_1(&"🔐 Signing transaction with Freighter...".into());

    let sign_result = sign_transaction_with_freighter(
        &join_response.transaction_xdr,
        &join_response.network_passphrase,
    )
    .await?;

    console::log_1(&"✅ Transaction signed successfully!".into());

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
        return Err(format!(
            "Failed to submit transaction: {}",
            response.status()
        ));
    }

    let result: TransactionResult = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse result: {}", e))?;

    console::log_1(&format!("🎉 Successfully joined contract! Hash: {}", result.hash).into());

    Ok(result)
}

/// Sign transaction with Freighter wallet
pub async fn sign_transaction_with_freighter(
    transaction_xdr: &str,
    network_passphrase: &str,
) -> Result<SignTransactionResponse, String> {
    // Check if Freighter is available
    let window = window().ok_or("No window object")?;
    let _freighter_api = window
        .get("freighterApi")
        .ok_or("Freighter API not found")?;

    // Create signing options
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
        &options,
        &"networkPassphrase".into(),
        &network_passphrase.into(),
    )
    .map_err(|_| "Failed to set network passphrase")?;

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

    Ok(SignTransactionResponse { signed_xdr })
}
