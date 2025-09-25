use crate::database::models::{GameInstance, PlayerStats, LeaderboardEntry};
use crate::database::connection::DbPool;
use sqlx::{Error as SqlxError};
use bigdecimal::ToPrimitive;

pub struct GameRepository;

impl GameRepository {
    pub async fn create_game_instance(
        pool: &DbPool,
        game_instance: &GameInstance,
    ) -> Result<GameInstance, SqlxError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO game_instances (
                user_id, game_session_id, player_username, player_wallet_address,
                player_result, player_score, opponent_score, duration_seconds, game_mode, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            RETURNING id, user_id, game_session_id, player_username, player_wallet_address,
                      player_result, player_score, opponent_score, duration_seconds, game_mode, created_at
            "#,
            game_instance.user_id,
            game_instance.game_session_id,
            game_instance.player_username,
            game_instance.player_wallet_address,
            game_instance.player_result,
            game_instance.player_score,
            game_instance.opponent_score,
            game_instance.duration_seconds,
            game_instance.game_mode
        )
        .fetch_one(pool)
        .await?;

        Ok(GameInstance {
            id: row.id,
            user_id: row.user_id,
            game_session_id: row.game_session_id,
            player_username: row.player_username,
            player_wallet_address: row.player_wallet_address,
            player_result: row.player_result,
            player_score: row.player_score,
            opponent_score: row.opponent_score,
            duration_seconds: row.duration_seconds,
            game_mode: row.game_mode.unwrap_or_else(|| "single_player_vs_ai".to_string()),
            created_at: row.created_at,
        })
    }

    pub async fn get_player_games(
        pool: &DbPool,
        wallet_address: &str,
        limit: Option<i64>,
    ) -> Result<Vec<GameInstance>, SqlxError> {
        let limit = limit.unwrap_or(50);

        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, game_session_id, player_username, player_wallet_address,
                   player_result, player_score, opponent_score, duration_seconds, game_mode, created_at
            FROM game_instances
            WHERE player_wallet_address = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            wallet_address,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| GameInstance {
                id: row.id,
                user_id: row.user_id,
                game_session_id: row.game_session_id,
                player_username: row.player_username,
                player_wallet_address: row.player_wallet_address,
                player_result: row.player_result,
                player_score: row.player_score,
                opponent_score: row.opponent_score,
                duration_seconds: row.duration_seconds,
                game_mode: row.game_mode.unwrap_or_else(|| "single_player_vs_ai".to_string()),
                created_at: row.created_at,
            })
            .collect())
    }

    pub async fn get_player_stats(
        pool: &DbPool,
        wallet_address: &str,
    ) -> Result<PlayerStats, SqlxError> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE player_result = 'Win') as wins,
                COUNT(*) FILTER (WHERE player_result = 'Loss') as losses,
                COUNT(*) FILTER (WHERE player_result = 'Draw') as draws,
                COUNT(*) as total_games,
                COALESCE(AVG(duration_seconds), 0) as avg_duration,
                COALESCE(AVG(player_score), 0) as avg_score
            FROM game_instances
            WHERE player_wallet_address = $1
            "#,
            wallet_address
        )
        .fetch_one(pool)
        .await?;

        Ok(PlayerStats {
            wins: row.wins.unwrap_or(0) as u32,
            losses: row.losses.unwrap_or(0) as u32,
            draws: row.draws.unwrap_or(0) as u32,
            total_games: row.total_games.unwrap_or(0) as u32,
            avg_duration: row.avg_duration
                .and_then(|d| d.to_f32())
                .unwrap_or(0.0),
            avg_score: row.avg_score
                .and_then(|d| d.to_f32())
                .unwrap_or(0.0),
        })
    }

    pub async fn get_recent_games(
        pool: &DbPool,
        limit: Option<i64>,
    ) -> Result<Vec<GameInstance>, SqlxError> {
        let limit = limit.unwrap_or(100);

        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, game_session_id, player_username, player_wallet_address,
                   player_result, player_score, opponent_score, duration_seconds, game_mode, created_at
            FROM game_instances
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| GameInstance {
                id: row.id,
                user_id: row.user_id,
                game_session_id: row.game_session_id,
                player_username: row.player_username,
                player_wallet_address: row.player_wallet_address,
                player_result: row.player_result,
                player_score: row.player_score,
                opponent_score: row.opponent_score,
                duration_seconds: row.duration_seconds,
                game_mode: row.game_mode.unwrap_or_else(|| "single_player_vs_ai".to_string()),
                created_at: row.created_at,
            })
            .collect())
    }

    pub async fn get_leaderboard(
        pool: &DbPool,
        limit: Option<i64>,
    ) -> Result<Vec<LeaderboardEntry>, SqlxError> {
        let limit = limit.unwrap_or(20);

        let rows = sqlx::query!(
            r#"
            SELECT
                player_username,
                player_wallet_address,
                COUNT(*) FILTER (WHERE player_result = 'Win') as wins,
                COUNT(*) FILTER (WHERE player_result = 'Loss') as losses,
                COUNT(*) FILTER (WHERE player_result = 'Draw') as draws,
                COUNT(*) as total_games
            FROM game_instances
            GROUP BY player_username, player_wallet_address
            ORDER BY wins DESC, total_games DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| LeaderboardEntry {
                username: row.player_username,
                wallet_address: row.player_wallet_address,
                wins: row.wins.unwrap_or(0) as u32,
                losses: row.losses.unwrap_or(0) as u32,
                draws: row.draws.unwrap_or(0) as u32,
                total_games: row.total_games.unwrap_or(0) as u32,
            })
            .collect())
    }
}

// PlayerStats and LeaderboardEntry moved to models.rs for better organization