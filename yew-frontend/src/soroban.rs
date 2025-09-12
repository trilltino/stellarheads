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


/// Sign transaction with Freighter wallet
pub async fn sign_transaction_with_freighter(
    transaction_xdr: &str,
    network_passphrase: &str,
) -> Result<SignTransactionResponse, String> {
    
    // Use the same Freighter API access as connect_wallet
    let window = window().ok_or("No window object")?;
    let freighter_api = js_sys::Reflect::get(window.as_ref(), &"freighterApi".into())
        .map_err(|_| "Freighter API not found")?;
        
    if freighter_api.is_undefined() || freighter_api.is_null() {
        return Err("Freighter API not available for signing".to_string());
    }

    // Create signing options
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
        &options,
        &"networkPassphrase".into(),
        &network_passphrase.into(),
    ).map_err(|_| "Failed to set network passphrase")?;

    // Get the signTransaction method from the API
    let sign_method = js_sys::Reflect::get(&freighter_api, &"signTransaction".into())
        .map_err(|_| "signTransaction method not found")?;
    
    let sign_function = sign_method.dyn_into::<js_sys::Function>()
        .map_err(|_| "signTransaction is not a function")?;

    // Call signTransaction with the XDR and options
    let args = js_sys::Array::new();
    args.push(&transaction_xdr.into());
    args.push(&options);

    let sign_promise = sign_function.apply(&freighter_api, &args)
        .map_err(|e| format!("Failed to call signTransaction: {:?}", e))?;
    
    let promise = sign_promise.dyn_into::<js_sys::Promise>()
        .map_err(|_| "signTransaction did not return a promise")?;

    let sign_result = JsFuture::from(promise)
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

