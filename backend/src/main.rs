use axum::{
    routing::{get, post},
    Router,
    response::Response,
    http::{StatusCode, Uri},
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use std::net::SocketAddr;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn spa_fallback(_uri: Uri) -> Result<Response, StatusCode> {
    match tokio::fs::read_to_string(FALLBACK_INDEX_PATH).await {
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

use backend::database::connection::create_pool;

const SERVER_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 3000);
const FRONTEND_DIST_PATH: &str = "../yew-frontend/dist";
const GAME_ASSETS_PATH: &str = "backend/static/game";
const FALLBACK_INDEX_PATH: &str = "../yew-frontend/dist/index.html";
use backend::handlers::{
    auth::register_guest,
    leaderboard::{
        join_leaderboard,
        record_game_result,
        submit_signed_transaction as submit_leaderboard_transaction,
        get_player_stats as get_leaderboard_stats,
        get_leaderboard,
        check_player_joined as check_leaderboard_joined,
        test_add_win
    },
    game_results::{
        store_game_result,
        get_player_stats as get_database_player_stats,
        get_player_games,
        get_database_leaderboard,
        get_recent_games,
    },
};

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

    let pool = match create_pool().await {
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

    let app = Router::new()
        .route("/api/auth/register-guest", post(register_guest))
        .route("/api/leaderboard/join", post(join_leaderboard))
        .route("/api/leaderboard/record", post(record_game_result))
        .route("/api/leaderboard/submit", post(submit_leaderboard_transaction))
        .route("/api/leaderboard/stats", get(get_leaderboard_stats))
        .route("/api/leaderboard", get(get_leaderboard))
        .route("/api/leaderboard/check", get(check_leaderboard_joined))
        .route("/api/leaderboard/test-add-win", get(test_add_win))
        .route("/api/games/store", post(store_game_result))
        .route("/api/games/player-stats", get(get_database_player_stats))
        .route("/api/games/player-games", get(get_player_games))
        .route("/api/games/leaderboard", get(get_database_leaderboard))
        .route("/api/games/recent", get(get_recent_games))

        .nest_service("/game", ServeDir::new(GAME_ASSETS_PATH))
        .nest_service("/static", ServeDir::new(FRONTEND_DIST_PATH))
        .fallback(spa_fallback)
        .with_state(pool)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(SERVER_ADDR);
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