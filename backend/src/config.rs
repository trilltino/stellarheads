use std::{env, net::SocketAddr};
use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub frontend_dist_path: String,
    pub game_assets_path: String,
    pub fallback_index_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| AppError::Config("SERVER_PORT must be a valid port number".to_string()))?;

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://stellar_user:stellar_pass@localhost:5432/stellar_heads".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port,
            frontend_dist_path: env::var("FRONTEND_DIST_PATH")
                .unwrap_or_else(|_| "../yew-frontend/dist".to_string()),
            game_assets_path: env::var("GAME_ASSETS_PATH")
                .unwrap_or_else(|_| "static/game".to_string()),
            fallback_index_path: env::var("FALLBACK_INDEX_PATH")
                .unwrap_or_else(|_| "../yew-frontend/dist/index.html".to_string()),
        })
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        format!("{}:{}", self.server_host, self.server_port)
            .parse()
            .map_err(|_| AppError::Config("Invalid server host:port combination".to_string()))
    }
}