use bevy::prelude::*;
use shared::protocol::ProtocolPlugin;

// Module declarations
pub mod client;
pub mod server;
pub mod renderer;
pub mod lobby;

// Re-export for main.rs
pub use client::ClientPlugin;
pub use server::ServerPlugin;
pub use renderer::RendererPlugin;
pub use lobby::{LobbyPlugin, LobbyState};

// ================= App Modes =================

#[derive(Debug, Clone)]
pub enum GameMode {
    Client,
    Server,
    HostClient, // Client + Server in same process
}

// ================= Shared Game Plugin =================

pub struct SharedGamePlugin;

impl Plugin for SharedGamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Add protocol
            .add_plugins(ProtocolPlugin)
            // Game state management
            .init_state::<AppState>()
            .init_state::<LobbyState>()
            // Add shared systems here (physics, etc.)
            .add_systems(Startup, setup_shared_systems);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    MainMenu,
    Lobby,
    InGame,
    GameOver,
}

fn setup_shared_systems() {
    println!("🔧 Shared game systems initialized");
}

// ================= CLI Arguments =================

#[derive(Debug, Clone)]
pub struct GameArgs {
    pub mode: GameMode,
    pub client_id: Option<u64>,
    pub server_port: u16,
    pub server_host: String,
    pub headless: bool,
}

impl Default for GameArgs {
    fn default() -> Self {
        Self {
            mode: GameMode::Client,
            client_id: None,
            server_port: 5000,
            server_host: "127.0.0.1".to_string(),
            headless: false,
        }
    }
}

pub fn parse_args() -> GameArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut game_args = GameArgs::default();
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "server" => {
                game_args.mode = GameMode::Server;
                game_args.headless = true; // Server is headless by default
            }
            "client" => {
                game_args.mode = GameMode::Client;
            }
            "host-client" => {
                game_args.mode = GameMode::HostClient;
            }
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    game_args.server_port = args[i + 1].parse().unwrap_or(5000);
                    i += 1;
                }
            }
            "-h" | "--host" => {
                if i + 1 < args.len() {
                    game_args.server_host = args[i + 1].clone();
                    i += 1;
                }
            }
            "-c" | "--client-id" => {
                if i + 1 < args.len() {
                    game_args.client_id = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--headless" => {
                game_args.headless = true;
            }
            _ => {}
        }
        i += 1;
    }
    
    println!("🚀 Game Args: {:?}", game_args);
    game_args
}