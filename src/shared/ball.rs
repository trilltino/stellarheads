use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Ball;

#[derive(Bundle)]
pub struct BallBundle {
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    collider_density: ColliderDensity,
    restitution: Restitution,
    friction: Friction,
    mass: Mass,
    ball: Ball,
}

impl BallBundle {
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
            friction: Friction::new(0.0),
            mass: Mass(10.0),
            ball: Ball,
            collider_density: ColliderDensity(11.3),
        }
    }
}

pub fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(BallBundle::new(
        &mut meshes,
        &mut materials,
        25.0,
        Color::srgb(1.0, 0.0, 0.0),
        Vec3::new(0.0, 100.0, 0.0),
    ));
}

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ball);
    }
}
