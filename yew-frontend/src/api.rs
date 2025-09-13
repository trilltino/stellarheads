use gloo_net::http::Request;
use serde_json::Value;
use shared::dto::{auth::Guest, user::SignUpResponse};

pub struct ApiClient {
    base_url: String,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
        }
    }
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn register_guest(&self, guest: Guest) -> Result<SignUpResponse, String> {
        let url = format!("{}/api/guest", self.base_url);
        
        let response = Request::post(&url)
            .header("content-type", "application/json")
            .json(&guest)
            .map_err(|e| format!("Request error: {e}"))?
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        
        if response.ok() {
            response
                .json::<SignUpResponse>()
                .await
                .map_err(|e| format!("Response parse error: {e}"))
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }

    pub async fn post(&self, endpoint: &str, data: Value) -> Result<String, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = Request::post(&url)
            .header("content-type", "application/json")
            .json(&data)
            .map_err(|e| format!("Request error: {e}"))?
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        
        if response.ok() {
            response.text().await.map_err(|e| format!("Response parse error: {e}"))
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }

    pub async fn get(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;
        
        if response.ok() {
            response.text().await.map_err(|e| format!("Response parse error: {e}"))
        } else {
            Err(format!("HTTP error: {}", response.status()))
        }
    }
}