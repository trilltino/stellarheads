use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub type DbPool = PgPool;

pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    // Load environment variables from backend/.env
    if let Err(e) = dotenvy::from_filename("backend/.env") {
        println!("Warning: Could not load backend/.env: {}", e);
        println!("Trying to load from current directory .env");
        dotenvy::dotenv().ok();
    }

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://stellar_user:stellar_pass@localhost:5432/stellar_heads".to_string());

    println!("Connecting to database: {}", database_url.replace("stellar_pass", "***"));

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    println!("✅ Database connected and migrations run successfully");
    Ok(pool)
}