use bevy::prelude::*;
use lightyear::prelude::*;
use crate::protocol::PlayerAction;

// ================= Input Handling System =================

/// Converts keyboard input to networked PlayerAction inputs
pub fn handle_input_system(
    mut input_manager: ResMut<InputManager<PlayerAction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Clear previous inputs
    input_manager.reset_all();
    
    // Handle movement
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        input_manager.press(PlayerAction::MoveLeft);
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        input_manager.press(PlayerAction::MoveRight);
    }
    
    // Handle jump
    if keyboard.pressed(KeyCode::Space) {
        input_manager.press(PlayerAction::Jump);
    }
    
    // Handle kick
    if keyboard.pressed(KeyCode::KeyX) {
        input_manager.press(PlayerAction::Kick);
    }
}

/// Applies networked inputs to player entities
pub fn apply_player_input_system(
    mut players: Query<(&mut Transform, &mut Velocity, &NetworkedPlayer), With<PlayerId>>,
    mut input_reader: EventReader<InputEvent<PlayerAction>>,
    time: Res<Time>,
) {
    for input_event in input_reader.read() {
        let player_id = input_event.client_id();
        
        // Find the player entity for this client
        for (mut transform, mut velocity, networked_player) in players.iter_mut() {
            // Apply the input to the player
            match input_event.input() {
                PlayerAction::MoveLeft => {
                    velocity.linvel.x = -networked_player.speed;
                }
                PlayerAction::MoveRight => {
                    velocity.linvel.x = networked_player.speed;
                }
                PlayerAction::Jump => {
                    if networked_player.is_grounded {
                        velocity.linvel.y = networked_player.jump_force;
                    }
                }
                PlayerAction::Kick => {
                    // Handle kick logic - will be implemented later
                    println!("Player {} kicked!", player_id);
                }
                PlayerAction::None => {
                    // Do nothing
                }
            }
        }
    }
}

// ================= Input Plugin =================

pub struct InputHandlingPlugin;

impl Plugin for InputHandlingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_input_system,
                apply_player_input_system,
            ),
        );
    }
}