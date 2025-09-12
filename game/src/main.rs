use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy::core_pipeline::bloom::Bloom;
mod egui_inspectorui;
mod shared;
mod server;
use egui_inspectorui::EguiInspector;
use shared::ball::{Ball, BallPlugin};
use shared::collision::CollisionPlugin;
use shared::goals::GoalPlugin;
use shared::ground::GroundPlugin;
use shared::player::{Player, AiPlayer, LocalPlayer, Speed, JumpForce, IsGrounded, CoyoteTime, PlayerPlugin};
use shared::scoring::ScoringPlugin;
use shared::state::AppState;
use shared::state_ui::StateUIPlugin;
use shared::ui::UIPlugin;

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "🌟 Stellar Heads".into(),
                    resolution: (1366.0, 768.0).into(), // Fixed standard laptop size
                    resizable: false, // Fixed window size
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            EguiPlugin::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
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
        .add_plugins(CollisionPlugin)
        .add_plugins(EguiInspector)
        .add_plugins(GoalPlugin)
        .add_plugins(GroundPlugin)
        .add_plugins(ScoringPlugin)
        .add_plugins(StateUIPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            ..default()
        },
        Bloom {
            prefilter: bevy::core_pipeline::bloom::BloomPrefilter {
                threshold: 0.6,
                threshold_softness: 0.2,
            },
            ..default()
        },
    ));
}
