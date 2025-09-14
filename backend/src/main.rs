use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use std::net::SocketAddr;

use backend::database::connection::create_pool;
use backend::handlers::{
    soroban::{create_join_transaction, check_player_joined, submit_signed_transaction},
    auth::register_guest,
    game::{submit_game_result, get_player_stats},
};

#[tokio::main]
async fn main() {
    // load env (root .env if present), then backend/.env inside create_pool
    dotenvy::dotenv().ok();

    let pool = create_pool()
        .await
        .expect("Failed to create database connection pool");

    let app = Router::new()
        // API routes
        .route("/join", post(create_join_transaction))
        .route("/check-joined", get(check_player_joined))
        .route("/submit-signed-transaction", post(submit_signed_transaction))
        .route("/api/guest", post(register_guest))
        .route("/api/game/result", post(submit_game_result))
        .route("/api/game/stats", get(get_player_stats))
        // Serve game WASM files at /game
        .nest_service("/game", ServeDir::new("../game/dist"))
        // Serve frontend files (should be last to catch all other routes)
        .fallback_service(ServeDir::new("../yew-frontend/dist"))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Backend server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}