use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use shared::dto::auth::UserType;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub wallet_address: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl User {
    pub fn get_user_type(&self) -> UserType {
        // Since all users in this table are guests by design
        UserType::Guest
    }
}