use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy::core_pipeline::bloom::Bloom;
mod shared;
mod rendering;

// use rendering::EguiInspector; // Disabled
use crate::shared::audio::music_system::GameAudioPlugin;
use shared::gameplay::{Ball, BallPlugin, CollisionPlugin, GoalPlugin, GroundPlugin, Player, AiPlayer, LocalPlayer, Speed, JumpForce, IsGrounded, CoyoteTime, PlayerPlugin};
use shared::scoring::ScoringPlugin;
use shared::{AppState, StateUIPlugin, UIPlugin};

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

fn main() {
    // WASM-specific setup for web deployment
    #[cfg(target_arch = "wasm32")]
    {
        // Set up panic hook for better error messages in browser console
        console_error_panic_hook::set_once();
        // Initialize console logging for WASM
        web_sys::console::log_1(&"🎮 Initializing Stellar Heads WASM...".into());
    }

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "🌟 Stellar Heads".into(),
                    resolution: (1366.0, 768.0).into(),
                    canvas: Some("#stellar-heads-canvas".into()), // Use the custom canvas from Yew frontend
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            EguiPlugin::default(),
        ))
        .init_asset::<bevy::audio::AudioSource>()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_state(AppState::LaunchMenu)
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
        // .add_plugins(EguiInspector)  // Disabled due to EGUI context issues
        .add_plugins(GameAudioPlugin)
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
