use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;
use shared::dto::common::ApiResponse;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),


    #[error("User not found")]
    UserNotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Stellar RPC error: {0}")]
    StellarRpc(String),

    #[error("Account error: {0}")]
    Account(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("XDR encoding error: {0}")]
    XdrEncoding(String),

    #[error("XDR decoding error: {0}")]
    XdrDecoding(String),

    #[error("Task execution error: {0}")]
    TaskExecution(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::ExternalService(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::EnvVar(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::StellarRpc(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Account(_) => StatusCode::BAD_REQUEST,
            AppError::Transaction(_) => StatusCode::BAD_REQUEST,
            AppError::XdrEncoding(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::XdrDecoding(_) => StatusCode::BAD_REQUEST,
            AppError::TaskExecution(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // Create a consistent error response using the ApiResponse format
        let detailed_message = match &self {
            AppError::InvalidInput(msg) => format!("Invalid input: {msg}"),
            AppError::ExternalService(msg) => format!("External service error: {msg}"),
            AppError::Internal(msg) => format!("Internal error: {msg}"),
            _ => self.to_string(),
        };

        let body = Json(ApiResponse::<()>::error(detailed_message));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;