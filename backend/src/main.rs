use axum::{
    http::{StatusCode, Uri},
    response::Response,
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use backend::{
    config::Config,
    database::create_pool,
    handlers::{
        auth::register_guest,
        game_results::{
            get_database_leaderboard, get_player_games, get_player_stats, get_recent_games,
            store_game_result,
        },
        contract::{
            generate_contract_xdr_handler, submit_contract_transaction_handler,
            get_leaderboard_handler, contract_health_handler, check_join_status_handler,
        },
        health,
    },
};

async fn spa_fallback(_uri: Uri) -> Result<Response, StatusCode> {
    let config = Config::from_env().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match tokio::fs::read_to_string(&config.fallback_index_path).await {
        Ok(content) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html")
            .body(content.into())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?),
        Err(e) => {
            error!("Failed to read index.html: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    let pool = match create_pool(&config).await {
        Ok(pool) => {
            info!("Database connection established");
            pool
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            error!("Make sure PostgreSQL is running and DATABASE_URL is set correctly");
            std::process::exit(1);
        }
    };

    let routes = create_routes(&config).with_state(pool);
    let app = routes.layer(CorsLayer::permissive());

    let addr = match config.socket_addr() {
        Ok(addr) => addr,
        Err(e) => {
            error!("Failed to parse socket address: {}", e);
            std::process::exit(1);
        }
    };
    info!("Backend server running on http://{}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to address {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server failed to start: {}", e);
        std::process::exit(1);
    }
}

fn create_routes(config: &Config) -> Router<sqlx::PgPool> {
    Router::new()
        // Health check
        .route("/health", get(health))
        // Authentication routes
        .route("/api/auth/register-guest", post(register_guest))
        // Game routes
        .route("/api/games/store", post(store_game_result))
        .route("/api/games/player-stats", get(get_player_stats))
        .route("/api/games/player-games", get(get_player_games))
        .route("/api/games/leaderboard", get(get_database_leaderboard))
        .route("/api/games/recent", get(get_recent_games))
        // Contract routes
        .route("/api/contract/generate-xdr", post(generate_contract_xdr_handler))
        .route("/api/contract/submit-transaction", post(submit_contract_transaction_handler))
        .route("/api/contract/join-status", get(check_join_status_handler))
        .route("/api/leaderboard", get(get_leaderboard_handler))
        .route("/api/contract/health", get(contract_health_handler))
        // Static file serving
        .nest_service("/game", ServeDir::new(&config.game_assets_path))
        .nest_service("/static", ServeDir::new(&config.frontend_dist_path))
        // SPA fallback for frontend routing
        .fallback(spa_fallback)
}