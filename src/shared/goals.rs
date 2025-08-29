use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Goal;

#[derive(Bundle)]
pub struct GoalBundle {
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
}

impl GoalBundle {
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
            rigid_body: RigidBody::Static,
            collider: Collider::circle(radius),
        }
    }
}

