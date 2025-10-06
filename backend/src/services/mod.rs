pub mod auth_service;
pub mod game_service;

// Soroban smart contract infrastructure
pub mod soroban;

// Re-export commonly used types
pub use auth_service::AuthService;
pub use game_service::GameService;
pub use soroban::ScalableContractManager;