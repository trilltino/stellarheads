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

// SPA fallback handler
async fn spa_fallback(_uri: Uri) -> Result<Response, StatusCode> {
    match tokio::fs::read_to_string("../yew-frontend/dist/index.html").await {
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
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Create database pool
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
        // Auth routes
        .route("/api/auth/register-guest", post(register_guest))
        // Blockchain Leaderboard API routes
        .route("/api/leaderboard/join", post(join_leaderboard))
        .route("/api/leaderboard/record", post(record_game_result))
        .route("/api/leaderboard/submit", post(submit_leaderboard_transaction))
        .route("/api/leaderboard/stats", get(get_leaderboard_stats))
        .route("/api/leaderboard", get(get_leaderboard))
        .route("/api/leaderboard/check", get(check_leaderboard_joined))
        .route("/api/leaderboard/test-add-win", get(test_add_win))
        // Database Game Results API routes
        .route("/api/games/store", post(store_game_result))
        .route("/api/games/player-stats", get(get_database_player_stats))
        .route("/api/games/player-games", get(get_player_games))
        .route("/api/games/leaderboard", get(get_database_leaderboard))
        .route("/api/games/recent", get(get_recent_games))

        // Serve game WASM files at /game
        .nest_service("/game", ServeDir::new("static/game"))
        // Serve static frontend assets
        .nest_service("/static", ServeDir::new("../yew-frontend/dist"))
        // SPA fallback for all other routes
        .fallback(spa_fallback)
        .with_state(pool)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Backend server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}