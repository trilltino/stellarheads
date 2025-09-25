use crate::database::connection::DbPool;
use crate::database::repositories::user_repository::UserRepository;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use tracing::{info, error};


use shared::dto::auth::Guest;
use shared::dto::user::{SignUpResponse, UserPublic};
use crate::database::models::User;

fn create_user_public(user: &User) -> UserPublic {
    UserPublic {
        id: user.id.to_string(),
        username: user.username.clone(),
        wallet_address: user.wallet_address.clone(),
        created_at: user.created_at.map_or("Unknown".to_string(), |dt| dt.to_string()),
    }
}

fn create_error_user_public() -> UserPublic {
    UserPublic {
        id: "error".to_string(),
        username: "Error".to_string(),
        wallet_address: "".to_string(),
        created_at: "".to_string(),
    }
}

pub async fn register_guest(
    State(pool): State<DbPool>,
    Json(req): Json<Guest>,
) -> (StatusCode, Json<SignUpResponse>) {
    info!("Received guest registration: username={}, wallet_address={}", req.username, req.wallet_address);
    
    match UserRepository::find_by_wallet_address(&pool, &req.wallet_address).await {
        Ok(Some(existing_user)) => {
            let user = if existing_user.username != req.username {
                match UserRepository::update_username(&pool, &req.wallet_address, &req.username).await {
                    Ok(updated_user) => updated_user,
                    Err(e) => {
                        error!("Failed to update username: {:?}", e);
                        existing_user
                    }
                }
            } else {
                existing_user
            };

            let user_public = create_user_public(&user);
            
            let resp = SignUpResponse {
                user: user_public,
                message: "Guest login successful!".to_string(),
            };
            (StatusCode::OK, Json(resp))
        }
        Ok(None) => {
            match UserRepository::create_guest(&pool, &req.username, &req.wallet_address).await {
                Ok(db_user) => {
                    let user_public = create_user_public(&db_user);
                    
                    let resp = SignUpResponse {
                        user: user_public,
                        message: "Guest created successfully!".to_string(),
                    };
                    (StatusCode::CREATED, Json(resp))
                }
                Err(e) => {
                    error!("Database error creating guest: {:?}", e);
                    let resp = SignUpResponse {
                        user: create_error_user_public(),
                        message: format!("Failed to create guest: {}", e),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(resp))
                }
            }
        }
        Err(e) => {
            error!("Database error finding user: {:?}", e);
            let resp = SignUpResponse {
                user: create_error_user_public(),
                message: format!("Database error: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(resp))
        }
    }
}