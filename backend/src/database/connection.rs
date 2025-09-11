use sqlx::{Pool, Postgres, PgPool};
use std::env;

pub type DbPool = Pool<Postgres>;

pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/stellar_heads".to_string());
    
    println!("Connecting to database: {}", database_url.replace("password", "***"));
    
    let pool = PgPool::connect(&database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    println!("✅ Database connected and migrations completed");
    Ok(pool)
}