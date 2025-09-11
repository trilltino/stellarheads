use crate::database::connection::DbPool;
use crate::database::models::user::User;
use shared::dto::auth::UserType;
use sqlx::Row;

pub struct UserRepository;

impl UserRepository {
    pub async fn create_user(
        pool: &DbPool,
        email: Option<String>,
        display_name: &str,
        password_hash: Option<String>,
        user_type: UserType,
        g_address: Option<String>,
        project_type: Option<String>,
        admin_type: Option<String>,
    ) -> Result<User, sqlx::Error> {
        let user_type_str = user_type.to_string();
        
        let result = sqlx::query!(
            r#"
            INSERT INTO users (email, display_name, password_hash, user_type, g_address, project_type, admin_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, email, display_name, password_hash, user_type, g_address, project_type, admin_type, created_at
            "#,
            email,
            display_name,
            password_hash,
            user_type_str,
            g_address,
            project_type,
            admin_type
        )
        .fetch_one(pool)
        .await?;

        Ok(User {
            id: result.id,
            email: result.email,
            display_name: result.display_name,
            password_hash: result.password_hash,
            user_type: result.user_type,
            g_address: result.g_address,
            project_type: result.project_type,
            admin_type: result.admin_type,
            created_at: result.created_at,
        })
    }

    pub async fn find_by_wallet_address(
        pool: &DbPool,
        wallet_address: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let result = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE g_address = $1",
            wallet_address
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn update_wallet_address(
        pool: &DbPool,
        user_id: i32,
        wallet_address: &str,
    ) -> Result<User, sqlx::Error> {
        let result = sqlx::query_as!(
            User,
            "UPDATE users SET g_address = $1 WHERE id = $2 RETURNING *",
            wallet_address,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}