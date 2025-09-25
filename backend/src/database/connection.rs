use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tracing::{info, warn};

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

pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    if let Err(e) = dotenvy::from_filename("backend/.env") {
        warn!("Could not load backend/.env: {}", e);
        info!("Trying to load from current directory .env");
        dotenvy::dotenv().ok();
    }

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://stellar_user:stellar_pass@localhost:5432/stellar_heads".to_string());

    info!("Connecting to database: {}", mask_password(&database_url));

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    info!("Database connected and migrations run successfully");
    Ok(pool)
}