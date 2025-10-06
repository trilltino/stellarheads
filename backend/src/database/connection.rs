use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, warn};
use crate::config::Config;

pub type DbPool = PgPool;

fn mask_password(url: &str) -> String {
    url.split('@')
        .enumerate()
        .map(|(i, part)| {
            if i == 0 && part.contains(':') {
                let mut parts: Vec<&str> = part.split(':').collect();
                if parts.len() >= 3 {
                    parts[2] = "***";
                }
                parts.join(":")
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("@")
}

pub async fn create_pool(config: &Config) -> Result<DbPool, sqlx::Error> {
    if let Err(e) = dotenvy::from_filename("backend/.env") {
        warn!("Could not load backend/.env: {}", e);
        info!("Trying to load from current directory .env");
        dotenvy::dotenv().ok();
    }

    info!("Connecting to database: {}", mask_password(&config.database_url));

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    info!("Database connected and migrations run successfully");
    Ok(pool)
}

pub async fn create_pool_with_default_config() -> Result<DbPool, sqlx::Error> {
    let config = Config::from_env()
        .map_err(|_| sqlx::Error::Configuration("Failed to load configuration".into()))?;
    create_pool(&config).await
}