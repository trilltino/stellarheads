use crate::database::connection::DbPool;
use crate::database::repositories::user_repository::UserRepository;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};


use shared::dto::auth::Guest;
use shared::dto::user::{SignUpResponse, UserPublic};

pub async fn register_guest(
    State(pool): State<DbPool>,
    Json(req): Json<Guest>,
) -> (StatusCode, Json<SignUpResponse>) {
    println!("Received guest registration: username={}, wallet_address={}", req.username, req.wallet_address);
    
    // Check if user already exists with this wallet address
    match UserRepository::find_by_wallet_address(&pool, &req.wallet_address).await {
        Ok(Some(existing_user)) => {
            // User already exists, update username if different
            let user = if existing_user.username != req.username {
                match UserRepository::update_username(&pool, &req.wallet_address, &req.username).await {
                    Ok(updated_user) => updated_user,
                    Err(e) => {
                        println!("Failed to update username: {:?}", e);
                        existing_user
                    }
                }
            } else {
                existing_user
            };

            let user_public = UserPublic {
                id: user.id.to_string(),
                username: user.username,
                wallet_address: user.wallet_address,
                created_at: user.created_at.map_or("Unknown".to_string(), |dt| dt.to_string()),
            };
            
            let resp = SignUpResponse {
                user: user_public,
                message: "Guest login successful!".to_string(),
            };
            (StatusCode::OK, Json(resp))
        }
        Ok(None) => {
            // Create new user
            match UserRepository::create_guest(&pool, &req.username, &req.wallet_address).await {
                Ok(db_user) => {
                    let user_public = UserPublic {
                        id: db_user.id.to_string(),
                        username: db_user.username,
                        wallet_address: db_user.wallet_address,
                        created_at: db_user.created_at.map_or("Unknown".to_string(), |dt| dt.to_string()),
                    };
                    
                    let resp = SignUpResponse {
                        user: user_public,
                        message: "Guest created successfully!".to_string(),
                    };
                    (StatusCode::CREATED, Json(resp))
                }
                Err(e) => {
                    println!("Database error creating guest: {:?}", e);
                    let error_user = UserPublic {
                        id: "error".to_string(),
                        username: "Error".to_string(),
                        wallet_address: "".to_string(),
                        created_at: "".to_string(),
                    };
                    let resp = SignUpResponse {
                        user: error_user,
                        message: format!("Failed to create guest: {}", e),
                    };
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(resp))
                }
            }
        }
        Err(e) => {
            println!("Database error finding user: {:?}", e);
            let error_user = UserPublic {
                id: "error".to_string(),
                username: "Error".to_string(),
                wallet_address: "".to_string(),
                created_at: "".to_string(),
            };
            let resp = SignUpResponse {
                user: error_user,
                message: format!("Database error: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(resp))
        }
    }
}