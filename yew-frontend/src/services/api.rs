use gloo_net::http::Request;
use shared::dto::{auth::Guest, user::SignUpResponse, common::ApiResponse};

#[derive(Default)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
        }
    }

    pub async fn register_guest(&self, guest: Guest) -> Result<SignUpResponse, String> {
        let url = if self.base_url.is_empty() {
            "http://localhost:3000/api/auth/register-guest".to_string()
        } else {
            format!("{}/api/auth/register-guest", self.base_url)
        };

        let response = Request::post(&url)
            .header("content-type", "application/json")
            .json(&guest)
            .map_err(|e| format!("Request error: {e}"))?
            .send()
            .await
            .map_err(|e| format!("Network error: {e}"))?;

        if response.ok() {
            let api_response: ApiResponse<SignUpResponse> = response
                .json()
                .await
                .map_err(|e| format!("Response parse error: {e}"))?;

            if api_response.success {
                api_response.data
                    .ok_or_else(|| "No data in successful response".to_string())
            } else {
                Err(api_response.message)
            }
        } else {
            // Try to parse error response
            match response.json::<ApiResponse<()>>().await {
                Ok(error_response) => Err(error_response.message),
                Err(_) => Err(format!("HTTP error: {}", response.status())),
            }
        }
    }
}