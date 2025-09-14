use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use wasm_bindgen::prelude::*;

mod shared;
mod rendering;

use rendering::EguiInspector;
use shared::gameplay::{Ball, BallPlugin, CollisionPlugin, GoalPlugin, GroundPlugin, Player, AiPlayer, LocalPlayer, Speed, JumpForce, IsGrounded, CoyoteTime, PlayerPlugin};
use shared::scoring::ScoringPlugin;
use shared::{AppState, StateUIPlugin, UIPlugin};

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

#[wasm_bindgen(start)]
pub fn main_js() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    run_game();
}

pub fn run_game() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#stellar-heads-canvas".to_owned()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            EguiPlugin::default(),
        ))
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
    commands.spawn(Camera2d);
}