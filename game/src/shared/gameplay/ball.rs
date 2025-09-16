use super::collision::CollisionLayers;
use crate::shared::AppState;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;


// ================= Ball =================

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct Ball {
    #[inspector(min = 0.0, max = 2.0, speed = 0.1)]
    pub bounce_multiplier: f32,
    #[inspector(min = 0.0, max = 1000.0)]
    pub max_speed: f32,
    #[inspector(min = 0.0, max = 50.0)]
    pub mass: f32,
    #[inspector(min = 0.0, max = 20.0)]
    pub gravity_scale: f32,
    #[inspector(min = 0.0, max = 1.0, speed = 0.01)]
    pub restitution: f32,
    #[inspector(min = 0.0, max = 1.0, speed = 0.01)]
    pub friction: f32,
    #[inspector(min = 10.0, max = 100.0)]
    pub radius: f32,
}

#[derive(Bundle)]
pub struct BallBundle {
    // visuals
    sprite: Sprite,
    transform: Transform,
    // physics
    rigid_body: RigidBody,
    collider: Collider,
    restitution: Restitution,
    friction: Friction,
    mass: Mass,
    gravity_scale: GravityScale,
    velocity: LinearVelocity,
    collider_density: ColliderDensity,
    layers: avian2d::prelude::CollisionLayers,
    // tag
    ball: Ball,
}

impl BallBundle {
    pub fn new(
        radius: f32,
        position: Vec3,
        ball_texture: Handle<Image>,
    ) -> Self {
        Self {
            sprite: Sprite {
                image: ball_texture,
                custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(radius),
            restitution: Restitution::new(0.8), // Less bouncy for better control
            friction: Friction::new(0.1),       // Slightly more friction
            mass: Mass(2.0),                    // Much lighter for responsiveness
            gravity_scale: GravityScale(12.0),   // Stronger gravity to keep it grounded
            velocity: LinearVelocity::ZERO,
            collider_density: ColliderDensity(1.0),
            layers: avian2d::prelude::CollisionLayers::new(
                CollisionLayers::BALL,
                CollisionLayers::GOAL | CollisionLayers::PLAYER | CollisionLayers::GROUND
            ),
            ball: Ball {
                bounce_multiplier: 0.8,
                max_speed: 400.0,              // Faster max speed
                mass: 2.0,
                gravity_scale: 12.0,
                restitution: 0.8,
                friction: 0.05,
                radius,
            },
        }
    }
}

fn cleanup_balls(
    mut commands: Commands,
    ball_query: Query<Entity, With<Ball>>,
) {
    for entity in ball_query.iter() {
        commands.entity(entity).despawn();
        println!("🗑️ Despawned ball entity: {:?}", entity);
    }
    println!("🧹 All balls cleaned up for new game");
}

pub fn spawn_ball(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let ball_radius = 24.0; // Increased size for better visibility and gameplay
    let spawn_height = 100.0; // Lower spawn height to prevent falling through

    // Load the ball texture
    let ball_texture = asset_server.load("ball/ball.png");

    let ball_entity = commands.spawn((
        BallBundle::new(
            ball_radius,
            Vec3::new(0.0, spawn_height, 0.0), // Center field, safer height
            ball_texture,
        ),
        Name::new("Soccer Ball"),
    )).id();

    println!("⚽ BALL SPAWNED: Entity {:?} at center field, height {} with radius {}",
             ball_entity, spawn_height, ball_radius);
    println!("   Using ball.png texture with proper scaling");
    println!("   Collision layers: BALL={} (collides with GOAL={}, PLAYER={}, GROUND={})",
             CollisionLayers::BALL, CollisionLayers::GOAL, CollisionLayers::PLAYER, CollisionLayers::GROUND);
}


// ================= Plugin =================

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Ball>()
            .add_systems(OnEnter(AppState::InGame), (cleanup_balls, spawn_ball).chain());
    }
}

