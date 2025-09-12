use crate::database::connection::DbPool;
use crate::database::models::user::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn create_guest(
        pool: &DbPool,
        username: &str,
        wallet_address: &str,
    ) -> Result<User, sqlx::Error> {
        let result = sqlx::query_as!(
            User,
            "INSERT INTO users (username, wallet_address) VALUES ($1, $2) RETURNING *",
            username,
            wallet_address
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    pub async fn find_by_wallet_address(
        pool: &DbPool,
        wallet_address: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let result = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE wallet_address = $1",
            wallet_address
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn update_username(
        pool: &DbPool,
        wallet_address: &str,
        username: &str,
    ) -> Result<User, sqlx::Error> {
        let result = sqlx::query_as!(
            User,
            "UPDATE users SET username = $1 WHERE wallet_address = $2 RETURNING *",
            username,
            wallet_address
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}