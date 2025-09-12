// src/main.rs

use backend::{create_app, create_pool};

#[tokio::main]
async fn main() {
    // Load .env early so DB config is available
    dotenvy::dotenv().ok();

    let pool = create_pool()
        .await
        .expect("failed to create database pool");

    let app = create_app(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to 0.0.0.0:3000");

    println!("server running on http://localhost:3000");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    // Ctrl+C to stop
    let _ = tokio::signal::ctrl_c().await;
    println!("shutdown received");
}

