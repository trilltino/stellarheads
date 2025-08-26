use avian2d::prelude::*;
use bevy::prelude::*;
mod shared;
use crate::shared::player::PlayerPlugin;
use shared::BallPlugin;

pub const FIXED_TIMESTEP_HZ: f64 = 60.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_plugins(BallPlugin)
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
