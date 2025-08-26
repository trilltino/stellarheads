use avian2d::prelude::*;
use bevy::prelude::*;

// Marker component
#[derive(Component)]
struct Player;

// Speed component
#[derive(Component)]
struct Speed(f32);

#[derive(Bundle)]
struct PlayerBundle {
    marker: Player,
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    restitution: Restitution,
    velocity: LinearVelocity,
    speed: Speed,
    mass: Mass,
}

impl PlayerBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        radius: f32,
        color: Color,
        position: Vec3,
    ) -> Self {
        Self {
            mesh: meshes.add(Circle::new(radius)).into(),
            material: materials.add(color).into(),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(radius),
            restitution: Restitution::new(0.8),
            velocity: LinearVelocity::ZERO, // Fixed: Initialize with ZERO
            speed: Speed(300.0),
            marker: Player,
            mass: Mass(1.0),
        }
    }
}

// Spawn the player
fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(PlayerBundle::new(
        &mut meshes,
        &mut materials,
        20.0,
        Color::srgb(0.2, 0.7, 0.9),
        Vec3::new(0.0, 0.0, 0.0),
    ));
}

// Player movement input -> updates LinearVelocity
fn player_physics_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut LinearVelocity, &Speed), With<Player>>, // Fixed: Use LinearVelocity
) {
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }

    for (mut vel, max) in &mut q {
        if dir != Vec2::ZERO {
            vel.0 = dir.normalize() * max.0; // Fixed: Access the Vec2 with .0
        } else {
            vel.0 = Vec2::ZERO;
        }
    }
}

// Plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, player_physics_input);
    }
}
