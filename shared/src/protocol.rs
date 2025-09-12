use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use crate::dto::user::Team;

// ================= Network Components =================

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub u64);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkedBall {
    pub radius: f32,
    pub mass: f32,
    pub bounce_multiplier: f32,
    pub max_speed: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkedPlayer {
    pub team: Team,
    pub speed: f32,
    pub jump_force: f32,
    pub is_grounded: bool,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkedGoal {
    pub team: Team,
}

// ================= Player Inputs =================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlayerAction {
    MoveLeft,
    MoveRight,
    Jump,
    Kick,
    None,
}

// ================= Network Messages =================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GoalScored {
    pub scoring_team: Team,
    pub goal_position: Vec3,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerReady {
    pub player_id: u64,
    pub team: Team,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameStarted;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchReset;

// ================= Lobby Messages =================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LobbyInfo {
    pub host_name: String,
    pub players_count: u8,
    pub max_players: u8,
    pub game_mode: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JoinLobbyRequest {
    pub player_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JoinLobbyResponse {
    pub success: bool,
    pub message: String,
    pub assigned_team: Option<Team>,
}

// ================= Protocol Configuration =================

#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Register components for networking
        app.register_type::<PlayerId>()
            .register_type::<NetworkedBall>()
            .register_type::<NetworkedPlayer>()
            .register_type::<NetworkedGoal>();
    }
}

// Network channel definitions
#[derive(Channel)]
pub struct PlayerInputChannel;

#[derive(Channel)]
pub struct GameEventChannel;

#[derive(Channel)]
pub struct LobbyChannel;

// Protocol setup function
pub fn protocol() -> MyProtocol {
    let mut protocol = MyProtocol::default();
    
    // Register components
    protocol.add_component::<PlayerId>(ComponentSyncMode::Full);
    protocol.add_component::<NetworkedBall>(ComponentSyncMode::Full);
    protocol.add_component::<NetworkedPlayer>(ComponentSyncMode::Full);
    protocol.add_component::<NetworkedGoal>(ComponentSyncMode::Simple);
    protocol.add_component::<Transform>(ComponentSyncMode::Full);
    
    // Register messages
    protocol.add_message::<GoalScored>(ChannelDirection::Bidirectional);
    protocol.add_message::<PlayerReady>(ChannelDirection::Bidirectional);
    protocol.add_message::<GameStarted>(ChannelDirection::ServerToClient);
    protocol.add_message::<MatchReset>(ChannelDirection::ServerToClient);
    protocol.add_message::<LobbyInfo>(ChannelDirection::ServerToClient);
    protocol.add_message::<JoinLobbyRequest>(ChannelDirection::ClientToServer);
    protocol.add_message::<JoinLobbyResponse>(ChannelDirection::ServerToClient);
    
    // Register inputs
    protocol.add_input::<PlayerAction>();
    
    // Configure channels
    protocol.add_channel::<PlayerInputChannel>(ChannelSettings {
        mode: ChannelMode::UnreliableUnordered,
        ..default()
    });
    
    protocol.add_channel::<GameEventChannel>(ChannelSettings {
        mode: ChannelMode::ReliableOrdered,
        ..default()
    });
    
    protocol.add_channel::<LobbyChannel>(ChannelSettings {
        mode: ChannelMode::ReliableOrdered,
        ..default()
    });
    
    protocol
}

// Type alias for our protocol
pub type MyProtocol = Protocol;