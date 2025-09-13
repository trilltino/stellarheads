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
        // Try to extract error message from various JS error formats
        let error_msg = if let Some(msg) = js_val.as_string() {
            msg
        } else if let Ok(error_obj) = js_val.clone().dyn_into::<js_sys::Error>() {
            error_obj.message().into()
        } else {
            // Try to convert to string for debugging
            match js_sys::JSON::stringify(&js_val.clone()) {
                Ok(json_str) => json_str.into(),
                Err(_) => format!("Unknown JS error: {:?}", js_val)
            }
        };

        // Check for specific error patterns
        let lower_msg = error_msg.to_lowercase();
        if lower_msg.contains("user") && (lower_msg.contains("reject") || lower_msg.contains("denied") || lower_msg.contains("cancel")) {
            FreighterError::UserRejected
        } else if lower_msg.contains("freighter") || lower_msg.contains("not found") || lower_msg.contains("undefined") {
            FreighterError::FreighterExtNotFound
        } else {
            FreighterError::JsExecutionError(error_msg)
        }
    }
}

fn get_freighter_api() -> Result<JsValue, FreighterError> {
    let window = window().ok_or(FreighterError::NoWindow)?;

    // Use a safer approach to check for freighterApi
    match Reflect::get(window.as_ref(), &JsValue::from_str("freighterApi")) {
        Ok(api) => {
            if api.is_undefined() || api.is_null() {
                web_sys::console::log_1(&JsValue::from_str("🔍 freighterApi is undefined/null"));
                Err(FreighterError::FreighterExtNotFound)
            } else {
                web_sys::console::log_1(&JsValue::from_str("✅ freighterApi found!"));
                Ok(api)
            }
        },
        Err(_) => {
            web_sys::console::log_1(&JsValue::from_str("❌ freighterApi not accessible"));
            Err(FreighterError::FreighterExtNotFound)
        }
    }
}

pub fn is_freighter_available() -> bool {
    get_freighter_api().is_ok()
}

pub async fn connect_wallet() -> Result<String, FreighterError> {
    web_sys::console::log_1(&JsValue::from_str("🚀 Starting Freighter connection..."));
    let api = get_freighter_api()?;

    // Add timeout to prevent infinite hang
    let timeout_duration = std::time::Duration::from_secs(30);
    web_sys::console::log_1(&JsValue::from_str(&format!("⏱️  Setting 30-second timeout for wallet connection")));

    web_sys::console::log_1(&JsValue::from_str("Requesting access..."));

    let request_access_method = Reflect::get(&api, &JsValue::from_str("requestAccess"))?;
    if request_access_method.is_function() {
        let function = request_access_method.dyn_into::<Function>()?;
        let promise = function.call0(&api)?;
        let promise = promise.dyn_into::<Promise>()?;
        match JsFuture::from(promise).await {
            Ok(_) => web_sys::console::log_1(&JsValue::from_str("Access granted!")),
            Err(e) => {
                web_sys::console::log_1(&JsValue::from_str("Access denied"));
                web_sys::console::log_1(&JsValue::from_str(&format!("Error details: {:?}", e)));
                // Log the raw error object for debugging
                web_sys::console::log_1(&e);
                return Err(FreighterError::from(e));
            }
        }
    }

    web_sys::console::log_1(&JsValue::from_str("Getting public key..."));
    let method_names = ["getPublicKey", "getUserInfo", "getAddress"];
    let mut get_public_key_method = JsValue::undefined();

    for method_name in &method_names {
        let method = Reflect::get(&api, &JsValue::from_str(method_name))?;

        if method.is_function() {
            web_sys::console::log_1(&JsValue::from_str(&format!("Found working method: {}", method_name)));
            get_public_key_method = method;
            break;
        }
    }

    if get_public_key_method.is_undefined() {
        return Err(FreighterError::JsExecutionError("No valid method found".to_string()));
    }

    let function = get_public_key_method.dyn_into::<Function>()?;
    let promise = function.call0(&api)?;
    let promise = promise.dyn_into::<Promise>()?;

    match JsFuture::from(promise).await {
        Ok(result) => {
            if let Some(public_key) = result.as_string() {
                web_sys::console::log_1(&JsValue::from_str(&format!("Got key: {}", public_key)));
                Ok(public_key)
            }
            else if let Ok(obj) = result.clone().dyn_into::<js_sys::Object>() {
                if let Ok(address) = Reflect::get(&obj, &JsValue::from_str("address")) {
                    if let Some(address_str) = address.as_string() {
                        web_sys::console::log_1(&JsValue::from_str(&format!("Got address: {}", address_str)));
                        Ok(address_str)
                    } else {
                        web_sys::console::log_1(&JsValue::from_str(&format!("Address not string: {:?}", address)));
                        Err(FreighterError::JsExecutionError("Address property is not a string".to_string()))
                    }
                } else {
                    web_sys::console::log_1(&JsValue::from_str(&format!("No address property found: {:?}", result)));
                    Err(FreighterError::JsExecutionError("No address property in result".to_string()))
                }
            } else {
                web_sys::console::log_1(&JsValue::from_str(&format!("Unknown result type: {:?}", result)));
                Err(FreighterError::JsExecutionError("getPublicKey returned unknown format".to_string()))
            }
        },
        Err(e) => {
            web_sys::console::log_1(&JsValue::from_str("getPublicKey failed"));
            web_sys::console::log_1(&JsValue::from_str(&format!("getPublicKey error details: {:?}", e)));
            web_sys::console::log_1(&e);
            Err(FreighterError::from(e))
        }
    }
}




