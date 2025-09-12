use bevy::prelude::*;
use lightyear::prelude::*;
use shared::protocol::*;
use crate::{AppState, GameArgs};
use std::collections::HashMap;

// ================= Server Plugin =================

pub struct ServerPlugin {
    pub args: GameArgs,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            // Server resources
            .init_resource::<ConnectedClients>()
            .init_resource::<GameLobby>()
            // Server systems
            .add_systems(Startup, setup_server)
            .add_systems(Update, (
                handle_client_connections,
                handle_client_messages,
                update_game_logic,
            ))
            .add_systems(OnEnter(AppState::InGame), spawn_game_entities);
    }
}

// ================= Server Resources =================

#[derive(Resource, Default)]
pub struct ConnectedClients {
    pub clients: HashMap<u64, ClientInfo>,
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: u64,
    pub name: String,
    pub team: Option<shared::dto::user::Team>,
    pub ready: bool,
}

#[derive(Resource)]
pub struct GameLobby {
    pub host_name: String,
    pub max_players: u8,
    pub current_players: u8,
    pub game_started: bool,
}

impl Default for GameLobby {
    fn default() -> Self {
        Self {
            host_name: "Stellar Heads Server".to_string(),
            max_players: 4,
            current_players: 0,
            game_started: false,
        }
    }
}

// ================= Server Systems =================

pub(crate) fn handle_new_client(trigger: Trigger<OnAdd, LinkOf>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .insert(ReplicationSender::new(
            SEND_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

fn handle_client_connections(
    mut clients: ResMut<ConnectedClients>,
    mut lobby: ResMut<GameLobby>,
) {
    // TODO: Handle client connect/disconnect when lightyear is integrated
}


fn handle_client_messages() {
    // TODO: Handle incoming client messages
}


fn update_game_logic(
    mut lobby: ResMut<GameLobby>,
    clients: Res<ConnectedClients>,
) {
    // Update lobby state
    lobby.current_players = clients.clients.len() as u8;
    
    // Check if we can start the game
    let ready_players = clients.clients.values()
        .filter(|client| client.ready)
        .count();
    
    if ready_players >= 2 && !lobby.game_started {
        println!("🎮 Starting game with {} players!", ready_players);
        lobby.game_started = true;
    }
}

fn spawn_game_entities(mut commands: Commands) {
    println!("🏟️ Spawning game entities on server");
    
    // Spawn ball
    commands.spawn((
        NetworkedBall {
            radius: 12.0,
            mass: 2.0,
            bounce_multiplier: 0.8,
            max_speed: 400.0,
        },
        Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
        Name::new("Networked Ball"),
    ));
    

    commands.spawn((
        NetworkedGoal {
            team: shared::dto::user::Team::Left,
        },
        Transform::from_translation(Vec3::new(-583.0, -265.0, 0.0)),
        Name::new("Left Goal"),
    ));
    
    commands.spawn((
        NetworkedGoal {
            team: shared::dto::user::Team::Right, 
        },
        Transform::from_translation(Vec3::new(583.0, -265.0, 0.0)),
        Name::new("Right Goal"),
    ));
}
