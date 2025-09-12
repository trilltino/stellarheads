use js_sys::{Function, Promise, Reflect};
use std::fmt;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

#[derive(Debug, Clone)]
pub enum FreighterError {
    FreighterExtNotFound,
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
            if error_msg.to_lowercase().contains("user") && error_msg.to_lowercase().contains("reject") {
                FreighterError::UserRejected
            } else if error_msg.contains("freighter") || error_msg.contains("not found") {
                FreighterError::FreighterExtNotFound
            } else {
                FreighterError::JsExecutionError(error_msg)
            }
        } else {
            FreighterError::JsExecutionError("Unknown JavaScript error".to_string())
        }
    }
}

fn get_freighter_api() -> Result<JsValue, FreighterError> {
    let window = window().ok_or(FreighterError::NoWindow)?;
    let api = Reflect::get(window.as_ref(), &JsValue::from_str("freighterApi"))?;
    if api.is_undefined() || api.is_null() {
        return Err(FreighterError::FreighterExtNotFound);
    }
    Ok(api)
}

pub fn is_freighter_available() -> bool {
    get_freighter_api().is_ok()
}

pub async fn connect_wallet() -> Result<String, FreighterError> {
    web_sys::console::log_1(&JsValue::from_str("🚀 Starting Freighter connection..."));
    
    // Wait for Freighter to be available (give it time to inject)
    let mut attempts = 0;
    let api = loop {
        match get_freighter_api() {
            Ok(api) => break api,
            Err(_) if attempts < 10 => {
                attempts += 1;
                web_sys::console::log_1(&JsValue::from_str(&format!("⏳ Waiting for Freighter... (attempt {})", attempts)));
                // Wait 100ms between attempts
                let promise = js_sys::Promise::resolve(&JsValue::UNDEFINED);
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                gloo::timers::future::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    };

    // First request access permission
    web_sys::console::log_1(&JsValue::from_str("🔐 Requesting access..."));
    let request_access_method = Reflect::get(&api, &JsValue::from_str("requestAccess"))?;

    if request_access_method.is_function() {
        let function = request_access_method.dyn_into::<Function>()?;
        let promise = function.call0(&api)?;
        let promise = promise.dyn_into::<Promise>()?;
        
        match JsFuture::from(promise).await {
            Ok(_) => {
                web_sys::console::log_1(&JsValue::from_str("✅ Access granted!"));
                
                // Now get the public key
                let get_public_key_method = Reflect::get(&api, &JsValue::from_str("getPublicKey"))?;
                if get_public_key_method.is_function() {
                    let function = get_public_key_method.dyn_into::<Function>()?;
                    let promise = function.call0(&api)?;
                    let promise = promise.dyn_into::<Promise>()?;
                    
                    match JsFuture::from(promise).await {
                        Ok(result) => {
                            if let Some(public_key) = result.as_string() {
                                web_sys::console::log_1(&JsValue::from_str(&format!("✅ Got public key: {}", public_key)));
                                return Ok(public_key);
                            } else {
                                return Err(FreighterError::JsExecutionError("Public key is not a string".to_string()));
                            }
                        },
                        Err(e) => {
                            return Err(FreighterError::from(e));
                        }
                    }
                } else {
                    return Err(FreighterError::JsExecutionError("getPublicKey is not a function".to_string()));
                }
            },
            Err(e) => {
                return Err(FreighterError::from(e));
            }
        }
    } else {
        return Err(FreighterError::JsExecutionError("requestAccess is not a function".to_string()));
    }
}




