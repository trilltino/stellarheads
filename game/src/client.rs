use bevy::prelude::*;
use lightyear::prelude::*;
use shared::protocol::*;
use shared::inputs::InputHandlingPlugin;
use crate::{AppState, GameArgs};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

// ================= Client Plugin =================

pub struct ClientPlugin {
    pub args: GameArgs,
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            // Input handling
            .add_plugins(InputHandlingPlugin)
            // Client-specific systems
            .add_systems(Startup, setup_client)
            .add_systems(Update, (
                handle_connection_events,
                handle_server_messages,
            ));
    }
}

// ================= Client Systems =================

fn setup_client() {
    println!("🎮 Client initialized");
}

fn handle_connection_events() {
    // TODO: Handle connection events when lightyear is integrated
}

fn handle_server_messages() {
    // TODO: Handle server messages when lightyear is integrated  
}
