use avian2d::prelude::*;
use bevy::prelude::*;
use wasm_bindgen::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

mod shared;
mod rendering;

use shared::gameplay::{Ball, BallPlugin, CollisionPlugin, GoalPlugin, GroundPlugin, Player, AiPlayer, LocalPlayer, Speed, JumpForce, IsGrounded, CoyoteTime, PlayerPlugin};
use shared::scoring::ScoringPlugin;
use shared::audio::music_system::GameAudioPlugin;
use shared::{AppState, StateUIPlugin, UIPlugin};

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

// Global guard to prevent multiple app initialization
static GAME_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[wasm_bindgen(start)]
pub fn main_js() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Check if game is already initialized
    if GAME_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"🎮 Game already initialized, skipping...".into());
        return;
    }

    run_game();
}

pub fn run_game() {

    #[cfg(target_arch = "wasm32")]
    let default_plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Stellar Heads".into(),
                resolution: (1366.0, 768.0).into(),
                canvas: Some("#stellar-heads-canvas".to_owned()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                resizable: false,
                ..default()
            }),
            ..default()
        })
;

    #[cfg(not(target_arch = "wasm32"))]
    let default_plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#stellar-heads-canvas".to_owned()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    });

    let mut app = App::new();

    // Add plugins conditionally for WASM vs native
    #[cfg(target_arch = "wasm32")]
    app.add_plugins((
        default_plugins,
        PhysicsPlugins::default(),
        // Audio disabled for WASM to avoid compatibility issues
    ));

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins((
        default_plugins,
        PhysicsPlugins::default(),
        bevy_egui::EguiPlugin::default(),
    ));

    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .init_asset::<bevy::audio::AudioSource>()
        .insert_state(AppState::InGame) // Skip menu for WASM - go straight to game
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
        .add_plugins(GoalPlugin)
        .add_plugins(GroundPlugin)
        .add_plugins(ScoringPlugin)
        .add_plugins(StateUIPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(GameAudioPlugin)
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::InGame), setup_game_background)
        .add_systems(OnExit(AppState::InGame), cleanup_game_background);


    app.run();
}

fn setup(mut commands: Commands) {
    use bevy::core_pipeline::bloom::Bloom;

    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_scale(Vec3::splat(1.5)), // Increase scale to zoom out and fit everything in 1366x768
        Bloom {
            prefilter: bevy::core_pipeline::bloom::BloomPrefilter {
                threshold: 0.6,
                threshold_softness: 0.2,
            },
            ..default()
        },
    ));
}

#[derive(Component)]
struct GameBackground;

fn setup_game_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let background_handle = asset_server.load("gamescreen/gamescreen.png");

    commands.spawn((
        Sprite::from_image(background_handle),
        Transform::from_xyz(0.0, 0.0, -10.0).with_scale(Vec3::splat(0.67)), // Scale to match camera zoom (1/1.5 = 0.67)
        GameBackground,
        Name::new("Game Background"),
    ));

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"🏟️ Game background loaded".into());

    #[cfg(not(target_arch = "wasm32"))]
    println!("🏟️ Game background loaded");
}

fn cleanup_game_background(
    mut commands: Commands,
    background_query: Query<Entity, With<GameBackground>>,
) {
    for entity in background_query.iter() {
        commands.entity(entity).despawn();
    }
}