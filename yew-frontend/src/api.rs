use gloo_net::http::Request;
use shared::dto::{auth::Guest, user::SignUpResponse};

pub struct ApiClient {
    base_url: String,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
        }
    }
}

impl ApiClient {

    pub async fn register_guest(&self, guest: Guest) -> Result<SignUpResponse, String> {
        let url = format!("{}/api/auth/register-guest", self.base_url);
        
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


}