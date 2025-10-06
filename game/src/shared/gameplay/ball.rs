use super::collision::CollisionLayers;
use crate::shared::{AppState, config::{GamePhysics, GameLayout}};

use avian2d::prelude::*;
use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[cfg_attr(not(target_arch = "wasm32"), derive(InspectorOptions))]
#[cfg_attr(not(target_arch = "wasm32"), reflect(InspectorOptions))]
pub struct Ball {
    #[cfg_attr(not(target_arch = "wasm32"), inspector(min = 0.0, max = 2.0, speed = 0.1))]
    pub bounce_multiplier: f32,
    #[cfg_attr(not(target_arch = "wasm32"), inspector(min = 0.0, max = 1000.0))]
    pub max_speed: f32,
    #[cfg_attr(not(target_arch = "wasm32"), inspector(min = 0.0, max = 50.0))]
    pub mass: f32,
    #[cfg_attr(not(target_arch = "wasm32"), inspector(min = 0.0, max = 20.0))]
    pub gravity_scale: f32,
    #[cfg_attr(not(target_arch = "wasm32"), inspector(min = 0.0, max = 1.0, speed = 0.01))]
    pub restitution: f32,
    #[cfg_attr(not(target_arch = "wasm32"), inspector(min = 0.0, max = 1.0, speed = 0.01))]
    pub friction: f32,
    #[cfg_attr(not(target_arch = "wasm32"), inspector(min = 10.0, max = 100.0))]
    pub radius: f32,
}

#[derive(Bundle)]
pub struct BallBundle {
    sprite: Sprite,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    restitution: Restitution,
    friction: Friction,
    mass: Mass,
    gravity_scale: GravityScale,
    velocity: LinearVelocity,
    collider_density: ColliderDensity,
    layers: avian2d::prelude::CollisionLayers,
    ball: Ball,
}

impl BallBundle {
    pub fn new(
        position: Vec3,
        ball_texture: Handle<Image>,
        physics: &GamePhysics,
        layout: &GameLayout,
    ) -> Self {
        Self {
            sprite: Sprite {
                image: ball_texture,
                custom_size: Some(Vec2::new(layout.ball_visual_size, layout.ball_visual_size)),
                ..default()
            },
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(layout.ball_physics_radius),
            restitution: Restitution::new(physics.ball_restitution),
            friction: Friction::new(physics.ball_friction),
            mass: Mass(physics.ball_mass),
            gravity_scale: GravityScale(physics.ball_gravity_scale),
            velocity: LinearVelocity::ZERO,
            collider_density: ColliderDensity(1.0),
            layers: avian2d::prelude::CollisionLayers::new(
                CollisionLayers::BALL,
                CollisionLayers::GOAL | CollisionLayers::PLAYER | CollisionLayers::GROUND
            ),
            ball: Ball {
                bounce_multiplier: physics.ball_bounce_multiplier,
                max_speed: physics.ball_max_speed,
                mass: physics.ball_mass,
                gravity_scale: physics.ball_gravity_scale,
                restitution: physics.ball_restitution,
                friction: physics.ball_friction,
                radius: layout.ball_physics_radius,
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
        #[cfg(not(target_arch = "wasm32"))]
        println!("🗑️ Despawned ball entity: {entity:?}");
    }
    #[cfg(not(target_arch = "wasm32"))]
    println!("🧹 All balls cleaned up for new game");
}

pub fn spawn_ball(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    physics: Res<GamePhysics>,
    layout: Res<GameLayout>,
) {
    let ball_texture = asset_server.load("ball/ball.png");
    let spawn_position = layout.ball_spawn();

    let ball_entity = commands.spawn((
        BallBundle::new(
            spawn_position,
            ball_texture,
            &physics,
            &layout,
        ),
        Name::new("Soccer Ball"),
    )).id();

    #[cfg(not(target_arch = "wasm32"))]
    println!(
        "⚽ BALL SPAWNED: Entity {:?} at {:?} with radius {}",
        ball_entity, spawn_position, layout.ball_physics_radius
    );
    #[cfg(not(target_arch = "wasm32"))]
    println!("   Using ball.png texture with proper scaling");
    #[cfg(not(target_arch = "wasm32"))]
    println!(
        "   Collision layers: BALL={} (collides with GOAL={}, PLAYER={}, GROUND={})",
        CollisionLayers::BALL, CollisionLayers::GOAL, CollisionLayers::PLAYER, CollisionLayers::GROUND
    );
}



pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Ball>()
            .add_systems(OnEnter(AppState::InGame), (cleanup_balls, spawn_ball).chain());
    }
}

