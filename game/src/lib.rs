use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::audio::AudioSource;
use wasm_bindgen::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod shared;
mod rendering;

use shared::config::{GameConfigPlugin, CameraConfig, BackgroundConfig};
use shared::gameplay::{
    Ball, BallPlugin, CollisionPlugin, GoalPlugin, GroundPlugin, Player, AiPlayer, LocalPlayer,
    Speed, JumpForce, IsGrounded, CoyoteTime, PlayerPlugin
};
use shared::scoring::ScoringPlugin;
use shared::audio::music_system::GameAudioPlugin;
use shared::{AppState, UIPlugin};

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
    create_app().run();
}

pub fn create_app() -> App {
    let mut app = App::new();

    add_plugins(&mut app);
    configure_app(&mut app);

    app
}

fn add_plugins(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    app.add_plugins((
        DefaultPlugins.set(create_window_plugin()),
        PhysicsPlugins::default(),
    ));

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins((
        DefaultPlugins.set(create_window_plugin()),
        PhysicsPlugins::default(),
        bevy_egui::EguiPlugin::default(),
    ));
}

fn create_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Stellar Heads".into(),
            resolution: (1366.0, 768.0).into(),
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#stellar-heads-canvas".into()),
            #[cfg(target_arch = "wasm32")]
            fit_canvas_to_parent: true,
            #[cfg(target_arch = "wasm32")]
            prevent_default_event_handling: false,
            resizable: false,
            ..default()
        }),
        ..default()
    }
}

fn configure_app(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .init_asset::<AudioSource>()
        .insert_state(AppState::InGame)
        .register_type::<Ball>()
        .register_type::<Player>()
        .register_type::<AiPlayer>()
        .register_type::<LocalPlayer>()
        .register_type::<Speed>()
        .register_type::<JumpForce>()
        .register_type::<IsGrounded>()
        .register_type::<CoyoteTime>()
        .add_plugins((
            GameConfigPlugin,  // Add game configuration resources
            BallPlugin,
            CollisionPlugin,
            GoalPlugin,
            GroundPlugin,
            ScoringPlugin,
            UIPlugin,
            PlayerPlugin,
            GameAudioPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::InGame), setup_game_background)
        .add_systems(OnExit(AppState::InGame), cleanup_game_background);
}

fn setup(mut commands: Commands, camera_config: Res<CameraConfig>) {
    use bevy::core_pipeline::bloom::Bloom;

    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_scale(Vec3::splat(camera_config.scale)),
        Bloom {
            prefilter: bevy::core_pipeline::bloom::BloomPrefilter {
                threshold: camera_config.bloom_threshold,
                threshold_softness: camera_config.bloom_threshold_softness,
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
    bg_config: Res<BackgroundConfig>,
) {
    let background_handle = asset_server.load(bg_config.asset_path.as_str());

    commands.spawn((
        Sprite::from_image(background_handle),
        Transform::from_xyz(0.0, 0.0, bg_config.z_depth).with_scale(Vec3::splat(bg_config.scale)),
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