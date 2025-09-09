use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod shared;
use shared::ball::BallPlugin;
use shared::player::PlayerPlugin;
use shared::state_ui::StateUIPlugin;
use shared::goals::GoalPlugin;
use shared::state::AppState;

use crate::shared::state_ui::launchui_system;

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default(), EguiPlugin::default()))
        .insert_state(AppState::LaunchMenu)
        .init_state::<AppState>() 
        .add_plugins(BallPlugin)
        .add_plugins(GoalPlugin)
        .add_plugins(StateUIPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::new(1500.0, 25.0)),
        Transform::from_xyz(0.0, -350.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(500.0, 25.0),
    ));
}
