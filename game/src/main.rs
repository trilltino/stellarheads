use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
mod egui_inspectorui;
mod shared;
use egui_inspectorui::EguiInspector;
use shared::ball::{Ball, BallPlugin};
use shared::goals::GoalPlugin;
use shared::player::{Player, AiPlayer, LocalPlayer, Speed, JumpForce, IsGrounded, CoyoteTime, PlayerPlugin};
use shared::state::AppState;
use shared::state_ui::StateUIPlugin;

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "🌟 Stellar Heads".into(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            EguiPlugin::default(),
        ))
        .insert_state(AppState::LaunchMenu)
        .init_state::<AppState>()
        .register_type::<Ball>()
        .register_type::<Player>()
        .register_type::<AiPlayer>()
        .register_type::<LocalPlayer>()
        .register_type::<Speed>()
        .register_type::<JumpForce>()
        .register_type::<IsGrounded>()
        .register_type::<CoyoteTime>()
        .add_plugins(BallPlugin)
        .add_plugins(EguiInspector)
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
