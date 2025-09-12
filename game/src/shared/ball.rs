use crate::shared::collision::CollisionLayers;
use crate::shared::state::AppState;

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
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
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
        color: Color,
        position: Vec3,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        Self {
            mesh: Mesh2d(meshes.add(Circle::new(radius))),
            material: MeshMaterial2d(materials.add(ColorMaterial::from(color.mix(&Color::WHITE, 0.5)))),
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

pub fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ball_radius = 12.0; // Perfect size for 1366x768 gameplay
    let spawn_height = 100.0; // Lower spawn height to prevent falling through
    
    let ball_entity = commands.spawn((
        BallBundle::new(
            ball_radius,
            Color::srgb(1.0, 0.0, 0.0),
            Vec3::new(0.0, spawn_height, 0.0), // Center field, safer height
            &mut meshes,
            &mut materials,
        ),
        Name::new("Player Ball"),
    )).id();
    
    println!("⚽ BALL SPAWNED: Entity {:?} at center field, height {} with radius {}", 
             ball_entity, spawn_height, ball_radius);
    println!("   Collision layers: BALL={} (collides with GOAL={}, PLAYER={}, GROUND={})", 
             CollisionLayers::BALL, CollisionLayers::GOAL, CollisionLayers::PLAYER, CollisionLayers::GROUND);
}


// ================= Plugin =================

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Ball>()
            .add_systems(OnEnter(AppState::InGame), spawn_ball);
    }
}

