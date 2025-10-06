use avian2d::prelude::*;
use bevy::prelude::*;
use crate::shared::config::{GameLayout, FieldPhysics};
use super::CollisionLayers;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ground);
    }
}

fn spawn_ground(
    mut commands: Commands,
    layout: Res<GameLayout>,
    field_physics: Res<FieldPhysics>,
) {
    spawn_continuous_field(&mut commands, &layout, &field_physics);
    spawn_field_walls(&mut commands, &layout, &field_physics);
    spawn_field_ceiling(&mut commands, &layout, &field_physics);
}

fn spawn_continuous_field(
    commands: &mut Commands,
    layout: &GameLayout,
    field_physics: &FieldPhysics,
) {
    commands.spawn((
        Sprite::from_color(
            Color::srgba(0.2, 0.8, 0.2, 0.0),
            Vec2::new(layout.field_width, layout.ground_height)
        ),
        Transform::from_xyz(0.0, layout.ground_level, 0.0),
        RigidBody::Static,
        Collider::rectangle(layout.field_width / 2.0, layout.ground_height / 2.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Restitution::new(field_physics.ground_restitution),
        Friction::new(field_physics.ground_friction),
        ColliderDensity(1000.0),
        Name::new("Continuous Field"),
    ));
}

fn spawn_field_walls(
    commands: &mut Commands,
    layout: &GameLayout,
    field_physics: &FieldPhysics,
) {
    let wall_positions = [
        layout.left_wall_x(),
        layout.right_wall_x(),
    ];
    let wall_names = ["Left Wall", "Right Wall"];

    for (&x_pos, &name) in wall_positions.iter().zip(wall_names.iter()) {
        let wall_entity = commands.spawn((
            Sprite::from_color(
                Color::srgba(0.8, 0.2, 0.2, 0.0),
                Vec2::new(layout.wall_thickness, layout.wall_height)
            ),
            Transform::from_xyz(x_pos, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(layout.wall_thickness / 2.0, layout.wall_height / 2.0),
            avian2d::prelude::CollisionLayers::new(
                CollisionLayers::GROUND,
                CollisionLayers::BALL | CollisionLayers::PLAYER
            ),
            Restitution::new(field_physics.wall_restitution),
            Friction::new(field_physics.wall_friction),
            ColliderDensity(1000.0),
            Name::new(name),
        )).id();

        #[cfg(not(target_arch = "wasm32"))]
        println!(
            "🧱 WALL SPAWNED: {} Entity {:?} at x={}, thickness={}, height={}",
            name, wall_entity, x_pos, layout.wall_thickness, layout.wall_height
        );
    }
}

fn spawn_field_ceiling(
    commands: &mut Commands,
    layout: &GameLayout,
    field_physics: &FieldPhysics,
) {
    commands.spawn((
        Transform::from_xyz(0.0, layout.ceiling_y, 0.0),
        RigidBody::Static,
        Collider::rectangle(layout.screen_width / 2.0, 10.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL
        ),
        Restitution::new(field_physics.ceiling_restitution),
        Name::new("Ceiling"),
    ));
}