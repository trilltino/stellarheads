use crate::database::connection::DbPool;
use crate::database::models::NewGameInstance;
use sqlx::{Error as SqlxError};
use bigdecimal::ToPrimitive;
use shared::dto::game::{GameInstance, PlayerStats, LeaderboardEntry};

pub struct GameRepository;

impl GameRepository {
    pub async fn create_game_instance(
        pool: &DbPool,
        new_game: NewGameInstance,
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
            new_game.user_id,
            new_game.game_session_id,
            new_game.player_username,
            new_game.player_wallet_address,
            new_game.player_result,
            new_game.player_score,
            new_game.opponent_score,
            new_game.duration_seconds,
            new_game.game_mode
        )
        .fetch_one(pool)
        .await?;

        Ok(GameInstance {
            id: row.id,
            game_session_id: row.game_session_id,
            player_username: row.player_username,
            player_wallet_address: row.player_wallet_address,
            player_result: row.player_result,
            player_score: row.player_score,
            opponent_score: row.opponent_score,
            duration_seconds: row.duration_seconds,
            game_mode: row.game_mode.unwrap_or_else(|| "single_player".to_string()),
            created_at: row.created_at,
        })
    }

    pub async fn get_player_games(
        pool: &DbPool,
        wallet_address: &str,
        limit: i64,
    ) -> Result<Vec<GameInstance>, SqlxError> {
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
                game_session_id: row.game_session_id,
                player_username: row.player_username,
                player_wallet_address: row.player_wallet_address,
                player_result: row.player_result,
                player_score: row.player_score,
                opponent_score: row.opponent_score,
                duration_seconds: row.duration_seconds,
                game_mode: row.game_mode.unwrap_or_else(|| "single_player".to_string()),
                created_at: row.created_at,
            })
            .collect())
    }

    pub async fn count_player_games(
        pool: &DbPool,
        wallet_address: &str,
    ) -> Result<i64, SqlxError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM game_instances WHERE player_wallet_address = $1",
            wallet_address
        )
        .fetch_one(pool)
        .await?;

        Ok(row.count.unwrap_or(0))
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
                COALESCE(AVG(player_score), 0) as avg_score,
                COALESCE(MAX(player_score), 0) as best_score
            FROM game_instances
            WHERE player_wallet_address = $1
            "#,
            wallet_address
        )
        .fetch_one(pool)
        .await?;

        let total_games = row.total_games.unwrap_or(0);
        let wins = row.wins.unwrap_or(0);
        let losses = row.losses.unwrap_or(0);
        let draws = row.draws.unwrap_or(0);

        let win_rate = if total_games > 0 {
            wins as f64 / total_games as f64
        } else {
            0.0
        };

        Ok(PlayerStats {
            total_games,
            wins,
            losses,
            draws,
            win_rate,
            average_score: row.avg_score.and_then(|d| d.to_f64()).unwrap_or(0.0),
            best_score: row.best_score.unwrap_or(0),
            total_playtime_seconds: row.avg_duration.and_then(|d| d.to_f64()).unwrap_or(0.0) * total_games as f64,
        })
    }

    pub async fn get_recent_games(
        pool: &DbPool,
        limit: i64,
    ) -> Result<Vec<GameInstance>, SqlxError> {
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
                game_session_id: row.game_session_id,
                player_username: row.player_username,
                player_wallet_address: row.player_wallet_address,
                player_result: row.player_result,
                player_score: row.player_score,
                opponent_score: row.opponent_score,
                duration_seconds: row.duration_seconds,
                game_mode: row.game_mode.unwrap_or_else(|| "single_player".to_string()),
                created_at: row.created_at,
            })
            .collect())
    }

    pub async fn count_total_games(pool: &DbPool) -> Result<i64, SqlxError> {
        let row = sqlx::query!("SELECT COUNT(*) as count FROM game_instances")
            .fetch_one(pool)
            .await?;

        Ok(row.count.unwrap_or(0))
    }

    pub async fn get_leaderboard(
        pool: &DbPool,
        limit: i64,
    ) -> Result<Vec<LeaderboardEntry>, SqlxError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                ROW_NUMBER() OVER (ORDER BY COUNT(*) FILTER (WHERE player_result = 'Win') DESC, COUNT(*) DESC) as rank,
                player_username,
                player_wallet_address,
                COUNT(*) FILTER (WHERE player_result = 'Win') as wins,
                COUNT(*) as total_games,
                COALESCE(MAX(player_score), 0) as best_score
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
            .map(|row| {
                let total_games = row.total_games.unwrap_or(0);
                let wins = row.wins.unwrap_or(0);
                let win_rate = if total_games > 0 {
                    wins as f64 / total_games as f64
                } else {
                    0.0
                };

                LeaderboardEntry {
                    rank: row.rank.unwrap_or(0),
                    username: row.player_username,
                    wallet_address: row.player_wallet_address,
                    wins,
                    total_games,
                    win_rate,
                    best_score: row.best_score.unwrap_or(0),
                }
            })
            .collect())
    }

    pub async fn count_total_players(pool: &DbPool) -> Result<i64, SqlxError> {
        let row = sqlx::query!(
            "SELECT COUNT(DISTINCT player_wallet_address) as count FROM game_instances"
        )
        .fetch_one(pool)
        .await?;

        Ok(row.count.unwrap_or(0))
    }
}