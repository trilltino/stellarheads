use crate::database::models::User;
use crate::database::connection::DbPool;
use sqlx::{Error as SqlxError};

pub struct UserRepository;

impl UserRepository {
    pub async fn create_guest(
        pool: &DbPool,
        username: &str,
        wallet_address: &str,
    ) -> Result<User, SqlxError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (username, wallet_address, created_at)
            VALUES ($1, $2, NOW())
            RETURNING id, username, wallet_address, created_at
            "#,
            username,
            wallet_address
        )
        .fetch_one(pool)
        .await?;

        Ok(User {
            id: row.id,
            username: row.username,
            wallet_address: row.wallet_address,
            created_at: row.created_at,
        })
    }

    pub async fn find_by_wallet_address(
        pool: &DbPool,
        wallet_address: &str,
    ) -> Result<Option<User>, SqlxError> {
        let row = sqlx::query!(
            "SELECT id, username, wallet_address, created_at FROM users WHERE wallet_address = $1",
            wallet_address
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.id,
                username: row.username,
                wallet_address: row.wallet_address,
                created_at: row.created_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_username(
        pool: &DbPool,
        wallet_address: &str,
        new_username: &str,
    ) -> Result<User, SqlxError> {
        let row = sqlx::query!(
            r#"
            UPDATE users
            SET username = $1
            WHERE wallet_address = $2
            RETURNING id, username, wallet_address, created_at
            "#,
            new_username,
            wallet_address
        )
        .fetch_one(pool)
        .await?;

        Ok(User {
            id: row.id,
            username: row.username,
            wallet_address: row.wallet_address,
            created_at: row.created_at,
        })
    }

    pub async fn find_by_id(
        pool: &DbPool,
        user_id: i32,
    ) -> Result<Option<User>, SqlxError> {
        let row = sqlx::query!(
            "SELECT id, username, wallet_address, created_at FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.id,
                username: row.username,
                wallet_address: row.wallet_address,
                created_at: row.created_at,
            }))
        } else {
            Ok(None)
        }
    }
}