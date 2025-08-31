use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
mod shared;
use crate::shared::player::PlayerPlugin;
use lightyear::prelude::*;
use shared::{BallPlugin, GoalPlugin, StateUIPlugin};

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

fn main() {
    App::new()
        .insert_resource(Gravity(Vec2::new(0.0, -980.0)))
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_plugins(BallPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(GoalPlugin)
        .add_systems(Startup, setup)
        .add_plugins(client::ClientPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        })
        .add_plugins(server::ServerPlugins {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        })
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Ground constants for easy tweaking
    const GROUND_WIDTH: f32 = 1500.0;
    const GROUND_HEIGHT: f32 = 50.0;
    const GROUND_Y: f32 = -350.0;

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.3, 0.3, 0.3),
            Vec2::new(GROUND_WIDTH, GROUND_HEIGHT),
        ),
        Transform::from_xyz(0.0, GROUND_Y, 0.0),
        RigidBody::Static,
        Collider::rectangle(GROUND_WIDTH, GROUND_HEIGHT),
    ));
}
