use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use wasm_bindgen::prelude::*;

mod egui_inspectorui;
mod shared;
use egui_inspectorui::EguiInspector;
use shared::ball::{Ball, BallPlugin};
use shared::collision::{CollisionPlugin, CollisionLayers};
use shared::goals::GoalPlugin;
use shared::player::{Player, AiPlayer, LocalPlayer, Speed, JumpForce, IsGrounded, CoyoteTime, PlayerPlugin};
use shared::scoring::ScoringPlugin;
use shared::state::AppState;
use shared::state_ui::StateUIPlugin;
use shared::ui::UIPlugin;

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
                    canvas: Some("#bevy".to_owned()),
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
        .add_plugins(ScoringPlugin)
        .add_plugins(StateUIPlugin)
        .add_plugins(UIPlugin)
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
        Collider::rectangle(750.0, 12.5),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Name::new("Ground"),
    ));
}