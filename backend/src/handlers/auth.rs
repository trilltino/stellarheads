use crate::database::connection::DbPool;
use crate::database::repositories::user_repository::UserRepository;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use shared::dto::auth::UserType;
use shared::dto::auth::Guest;
use shared::dto::user::{SignUpResponse, UserPublic};

use crate::database::connection::DbPool;
use crate::database::repositories::user_repository::UserRepository;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use shared::dto::auth::UserType;
use shared::dto::auth::Guest;
use shared::dto::user::{SignUpResponse, UserPublic};

pub async fn register_guest(
    State(pool): State<DbPool>,
    Json(req): Json<Guest>,
) -> (StatusCode, Json<SignUpResponse>) {
    println!("Received guest user: {:?}", req);
    match UserRepository::create_user(
        &pool,
        None,
        &req.username,
        None,
        UserType::Guest,
        req.wallet_address,
        None,
        None,
    )
    .await
    {
        Ok(db_user) => {
            let user = UserPublic {
                id: db_user.id.to_string(),
                email: db_user.email,
                display_name: db_user.display_name,
                created_at: db_user.created_at.to_string(),
                user_type: UserType::Guest,
                g_address: db_user.g_address,
                project_type: db_user.project_type,
                admin_type: db_user.admin_type,
            };
            let resp = SignUpResponse {
                user,
                message: "Guest created successfully!".into(),
            };
            (StatusCode::CREATED, Json(resp))
        }
        Err(e) => {
            println!("Database error: {:?}", e);
            let user = UserPublic {
                id: "error".into(),
                email: None,
                display_name: "Error".into(),
                created_at: "".into(),
                user_type: UserType::Guest,
                g_address: None,
                project_type: None,
                admin_type: None,
            };
            let resp = SignUpResponse {
                user,
                message: format!("Failed to create user: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(resp))
        }
    }
}