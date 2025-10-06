use crate::database::connection::DbPool;
use crate::database::models::NewGameInstance;
use crate::database::repositories::{game_repository::GameRepository, user_repository::UserRepository};
use crate::error::{AppError, Result};
use crate::services::soroban::client::{generate_leaderboard_xdr, ContractConfig};
use shared::dto::game::{
    StoreGameResultRequest, StoreGameResultResponse, PlayerStatsQuery, PlayerStats,
    PlayerGamesQuery, GameInstance, LeaderboardEntry, ContractXdrInfo,
};
use shared::dto::contract::LeaderboardFunction;
use shared::dto::common::PaginatedResponse;
use tracing::{info, warn};

pub struct GameService;

impl GameService {
    pub async fn store_game_result(
        pool: &DbPool,
        request: StoreGameResultRequest,
    ) -> Result<StoreGameResultResponse> {
        info!("Storing game result for player: {}", request.player_username);

        // Validate game result
        if !["Win", "Loss", "Draw"].contains(&request.player_result.as_str()) {
            return Err(AppError::InvalidInput(
                "player_result must be one of: Win, Loss, Draw".to_string()
            ));
        }

        // Find or create user
        let user = match UserRepository::find_by_wallet_address(pool, &request.player_wallet_address).await? {
            Some(user) => user,
            None => {
                // Auto-create user if they don't exist
                UserRepository::create_guest(pool, &request.player_username, &request.player_wallet_address).await?
            }
        };

        // Store data we need for contract XDR before moving into new_game
        let is_win = request.player_result == "Win";
        let player_wallet = request.player_wallet_address.clone();
        let player_name = request.player_username.clone();

        // Store the game result
        let new_game = NewGameInstance {
            user_id: Some(user.id),
            game_session_id: request.game_session_id,
            player_username: request.player_username,
            player_wallet_address: request.player_wallet_address,
            player_result: request.player_result,
            player_score: request.player_score,
            opponent_score: request.opponent_score,
            duration_seconds: request.duration_seconds,
            game_mode: request.game_mode.unwrap_or_else(|| "single_player".to_string()),
        };

        let game_instance = GameRepository::create_game_instance(pool, new_game).await?;

        // Generate contract XDR - check join status and generate appropriate XDR
        let contract_xdr = {
            info!("🔍 Checking player join status for contract interaction");

            let config = ContractConfig::default();
            let config_clone = config.clone();

            // Check if player has joined the leaderboard
            let has_joined_function = LeaderboardFunction::HasJoined {
                player: player_wallet.clone()
            };

            info!("📞 Calling has_joined for player: {}", player_wallet);

            // Clone variables for the spawned task
            let config_for_task = config.clone();
            let player_wallet_for_task = player_wallet.clone();

            // First check if player has joined
            let join_status_result = tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    // For now, we'll assume they need to join - in a real implementation
                    // you'd call the contract to check. This is a simplified approach.
                    generate_leaderboard_xdr(&config_for_task, &player_wallet_for_task, &has_joined_function).await
                })
            }).await;

            match join_status_result {
                Ok(Ok(_)) => {
                    info!("✅ Player join status checked successfully");
                    // For simplicity, we'll generate join XDR for first-time players
                    // In production, you'd parse the contract response to determine join status

                    if is_win {
                        info!("🏆 Win detected! Player needs to join first, generating join XDR");

                        let join_function = LeaderboardFunction::Join {
                            player: player_wallet.clone()
                        };

                        // Clone variables for the second spawned task
                        let config_for_join = config_clone.clone();
                        let player_wallet_for_join = player_wallet.clone();

                        let join_xdr_result = tokio::task::spawn_blocking(move || {
                            tokio::runtime::Handle::current().block_on(async move {
                                generate_leaderboard_xdr(&config_for_join, &player_wallet_for_join, &join_function).await
                            })
                        }).await;

                        match join_xdr_result {
                            Ok(Ok(xdr)) => {
                                info!("✅ Join XDR generated successfully");
                                info!("🔍 Join XDR preview: {}...{}", &xdr[0..50.min(xdr.len())], &xdr[xdr.len().saturating_sub(50)..]);
                                Some(ContractXdrInfo {
                                    xdr,
                                    function_name: "join".to_string(),
                                    description: format!("Join leaderboard for player {}", player_name),
                                    network_passphrase: config_clone.network_passphrase.clone(),
                                })
                            },
                            Ok(Err(e)) => {
                                warn!("❌ Failed to generate join XDR: {}", e);
                                None
                            },
                            Err(join_error) => {
                                warn!("❌ Join XDR generation task failed: {}", join_error);
                                None
                            }
                        }
                    } else {
                        info!("📊 Game result recorded (no win), no contract interaction needed");
                        None
                    }
                },
                Ok(Err(e)) => {
                    warn!("❌ Failed to check join status: {}", e);
                    None
                },
                Err(join_error) => {
                    warn!("❌ Join status check task failed: {}", join_error);
                    None
                }
            }
        };

        Ok(StoreGameResultResponse {
            game_id: game_instance.id,
            contract_xdr,
        })
    }

    pub async fn get_player_stats(
        pool: &DbPool,
        query: PlayerStatsQuery,
    ) -> Result<PlayerStats> {
        let stats = GameRepository::get_player_stats(pool, &query.wallet_address).await?;
        Ok(stats)
    }

    pub async fn get_player_games(
        pool: &DbPool,
        query: PlayerGamesQuery,
    ) -> Result<PaginatedResponse<GameInstance>> {
        let limit = query.limit.unwrap_or(20).min(100); // Cap at 100
        let offset = 0; // For now, simple pagination

        let games = GameRepository::get_player_games(pool, &query.wallet_address, limit).await?;
        let total = GameRepository::count_player_games(pool, &query.wallet_address).await?;

        Ok(PaginatedResponse::new(games, total, limit, offset))
    }

    pub async fn get_leaderboard(
        pool: &DbPool,
        limit: Option<i64>,
    ) -> Result<PaginatedResponse<LeaderboardEntry>> {
        let limit = limit.unwrap_or(20).min(100); // Cap at 100
        let offset = 0;

        let entries = GameRepository::get_leaderboard(pool, limit).await?;
        let total = GameRepository::count_total_players(pool).await?;

        Ok(PaginatedResponse::new(entries, total, limit, offset))
    }

    pub async fn get_recent_games(
        pool: &DbPool,
        limit: Option<i64>,
    ) -> Result<PaginatedResponse<GameInstance>> {
        let limit = limit.unwrap_or(20).min(100); // Cap at 100
        let offset = 0;

        let games = GameRepository::get_recent_games(pool, limit).await?;
        let total = GameRepository::count_total_games(pool).await?;

        Ok(PaginatedResponse::new(games, total, limit, offset))
    }

    pub async fn get_total_players(pool: &DbPool) -> Result<i64> {
        GameRepository::count_total_players(pool).await.map_err(AppError::Database)
    }
}