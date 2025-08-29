use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Ball;

#[derive(Bundle)]
pub struct BallBundle {
    sprite: Sprite,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    collider_density: ColliderDensity,
    restitution: Restitution,
    friction: Friction,
    mass: Mass,
    gravity_scale: GravityScale,
    velocity: LinearVelocity,
    ball: Ball,
}

impl BallBundle {
    pub fn new(radius: f32, color: Color, position: Vec3) -> Self {
        Self {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(radius),
            restitution: Restitution::new(0.8), // Bouncy ball
            friction: Friction::new(0.1),       // Low friction for rolling
            mass: Mass(1.0),                    // Lighter for more realistic physics
            gravity_scale: GravityScale(1.0),   // Normal gravity
            velocity: LinearVelocity::ZERO,
            collider_density: ColliderDensity(1.0), // More realistic density
            ball: Ball,
        }
    }
}

pub fn spawn_ball(mut commands: Commands) {
    commands.spawn(BallBundle::new(
        25.0,
        Color::srgb(1.0, 0.0, 0.0),
        Vec3::new(0.0, 200.0, 0.0), // Start higher up
    ));
}

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ball);
    }
}

