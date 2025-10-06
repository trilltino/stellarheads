/// Idiomatic Freighter wallet integration using direct WASM bindings
///
/// This module provides a clean, type-safe interface to the Freighter wallet
/// using proper WASM bindings and robust error handling.

use js_sys::{Function, Promise, Reflect};
use std::fmt;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, console};

#[derive(Debug, Clone)]
pub enum FreighterError {
    FreighterExtNotFound,
    NotAFunction(String),
    JsExecutionError(String),
    NoWindow,
    UserRejected,
}

impl fmt::Display for FreighterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FreighterError::FreighterExtNotFound => {
                write!(f, "Freighter wallet extension not found. Install from https://freighter.app/")
            }
            FreighterError::NotAFunction(method) => {
                write!(f, "Property {method} exists but is not a callable function")
            }
            FreighterError::JsExecutionError(msg) => {
                write!(f, "JavaScript error: {msg}")
            }
            FreighterError::NoWindow => write!(f, "Window object not available"),
            FreighterError::UserRejected => write!(f, "User rejected the connection request"),
        }
    }
}

impl std::error::Error for FreighterError {}

impl From<JsValue> for FreighterError {
    fn from(js_val: JsValue) -> Self {
        if let Some(error_msg) = js_val.as_string() {
            let error_lower = error_msg.to_lowercase();
            if error_lower.contains("user") && error_lower.contains("reject") {
                FreighterError::UserRejected
            } else if error_lower.contains("freighter") || error_lower.contains("not found") {
                FreighterError::FreighterExtNotFound
            } else {
                FreighterError::JsExecutionError(error_msg)
            }
        } else {
            FreighterError::JsExecutionError("Unknown JavaScript error".to_string())
        }
    }
}

// WASM bindings for official Freighter API from CDN
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "freighterApi"])]
    fn isConnected() -> Promise;

    #[wasm_bindgen(js_namespace = ["window", "freighterApi"])]
    fn requestAccess() -> Promise;

    #[wasm_bindgen(js_namespace = ["window", "freighterApi"])]
    fn getAddress() -> Promise;

    #[wasm_bindgen(js_namespace = ["window", "freighterApi"])]
    fn getPublicKey() -> Promise;

    #[wasm_bindgen(js_namespace = ["window", "freighterApi"])]
    fn signTransaction(xdr: &str, opts: &JsValue) -> Promise;
}

/// Get the Freighter API object from the CDN-loaded library
fn get_freighter_api() -> Result<JsValue, FreighterError> {
    let window = window().ok_or(FreighterError::NoWindow)?;

    // The Freighter API is loaded from CDN and available at window.freighterApi
    if let Ok(api) = Reflect::get(window.as_ref(), &JsValue::from_str("freighterApi")) {
        if !api.is_undefined() && !api.is_null() {
            return Ok(api);
        }
    }
    Err(FreighterError::FreighterExtNotFound)
}

/// Check if Freighter is available and installed
pub async fn is_freighter_available() -> bool {
    let api = match get_freighter_api() {
        Ok(api) => api,
        Err(_) => return false,
    };

    let is_connected = Reflect::get(&api, &JsValue::from_str("isConnected"))
        .unwrap_or(JsValue::UNDEFINED);

    if is_connected.is_function() {
        if let Some(func) = is_connected.dyn_into::<Function>().ok() {
            if let Some(promise) = func.call0(&api).ok() {
                if let Ok(promise) = promise.dyn_into::<Promise>() {
                    if let Ok(result) = JsFuture::from(promise).await {
                        if let Ok(obj) = result.dyn_into::<js_sys::Object>() {
                            if let Ok(connected) = Reflect::get(&obj, &JsValue::from_str("isConnected")) {
                                return connected.as_bool().unwrap_or(false);
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Connect to Freighter wallet and get the user's public key
pub async fn connect_wallet() -> Result<String, FreighterError> {
    let api = get_freighter_api()?;

    // Request access permission
    let request_access_method = Reflect::get(&api, &JsValue::from_str("requestAccess"))?;

    if request_access_method.is_function() {
        let function = request_access_method.dyn_into::<Function>()?;
        let promise = function.call0(&api)?;
        let promise = promise.dyn_into::<Promise>()?;

        match JsFuture::from(promise).await {
            Ok(_) => {},
            Err(e) => return Err(FreighterError::from(e)),
        }
    }

    // Get public key using multiple fallback methods
    let method_names = ["getPublicKey", "getUserInfo", "getAddress"];

    for method_name in &method_names {
        if let Ok(method) = Reflect::get(&api, &JsValue::from_str(method_name)) {
            if method.is_function() {
                let function = method.dyn_into::<Function>()?;
                let promise = function.call0(&api)?;
                let promise = promise.dyn_into::<Promise>()?;

                match JsFuture::from(promise).await {
                    Ok(result) => {
                        // Try to extract public key from various response formats
                        if let Some(public_key) = result.as_string() {
                            console::log_1(&"✅ Freighter connected".into());
                            return Ok(public_key);
                        } else if let Ok(obj) = result.clone().dyn_into::<js_sys::Object>() {
                            // Try different property names
                            for prop in &["address", "publicKey", "account"] {
                                if let Ok(addr) = Reflect::get(&obj, &JsValue::from_str(prop)) {
                                    if let Some(addr_str) = addr.as_string() {
                                        console::log_1(&"✅ Freighter connected".into());
                                        return Ok(addr_str);
                                    }
                                }
                            }
                        }
                    }
                    Err(_e) => continue,
                }
            }
        }
    }

    Err(FreighterError::JsExecutionError("No working method found to get public key".to_string()))
}

/// Sign a transaction XDR with Freighter
pub async fn sign_transaction(xdr: &str, network_passphrase: &str) -> Result<String, FreighterError> {
    let api = get_freighter_api()?;

    let sign_method = Reflect::get(&api, &JsValue::from_str("signTransaction"))?;
    if !sign_method.is_function() {
        return Err(FreighterError::NotAFunction("signTransaction".to_string()));
    }

    // Create options object for signing
    let opts = js_sys::Object::new();
    Reflect::set(&opts, &JsValue::from_str("networkPassphrase"), &JsValue::from_str(network_passphrase))?;

    let function = sign_method.dyn_into::<Function>()?;
    let args = js_sys::Array::new();
    args.push(&JsValue::from_str(xdr));
    args.push(&opts.into());

    let promise = function.apply(&api, &args)?;
    let promise = promise.dyn_into::<Promise>()?;

    match JsFuture::from(promise).await {
        Ok(result) => {
            // Extract signed XDR from result
            if let Some(signed_xdr) = result.as_string() {
                console::log_1(&"✅ Transaction signed".into());
                Ok(signed_xdr)
            } else if let Ok(obj) = result.clone().dyn_into::<js_sys::Object>() {
                // Try different property names for signed XDR
                for prop in &["signedTxXdr", "signedXdr", "xdr", "result"] {
                    if let Ok(xdr_val) = Reflect::get(&obj, &JsValue::from_str(prop)) {
                        if let Some(xdr_str) = xdr_val.as_string() {
                            console::log_1(&"✅ Transaction signed".into());
                            return Ok(xdr_str);
                        }
                    }
                }
                Err(FreighterError::JsExecutionError("Signed XDR not found in response".to_string()))
            } else {
                Err(FreighterError::JsExecutionError("Invalid response format from signTransaction".to_string()))
            }
        }
        Err(e) => Err(FreighterError::from(e))
    }
}