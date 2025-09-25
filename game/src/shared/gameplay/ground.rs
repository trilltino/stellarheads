use avian2d::prelude::*;
use bevy::prelude::*;
use super::CollisionLayers;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ground);
    }
}

fn spawn_ground(mut commands: Commands) {
    spawn_continuous_field(&mut commands);
    spawn_field_walls(&mut commands);
    spawn_field_ceiling(&mut commands);
}

fn spawn_continuous_field(commands: &mut Commands) {
    let field_width = 5000.0; 
    let field_height = 50.0;
    let ground_y = -350.0; 
    
    commands.spawn((
        Sprite::from_color(Color::srgba(0.2, 0.8, 0.2, 0.0), Vec2::new(field_width, field_height)),
        Transform::from_xyz(0.0, ground_y, 0.0),
        RigidBody::Static,
        Collider::rectangle(field_width / 2.0, field_height / 2.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Restitution::new(0.1),
        Friction::new(0.9),
        ColliderDensity(1000.0),
        Name::new("Continuous Field"),
    ));
}

fn spawn_field_walls(commands: &mut Commands) {
    let wall_height = 1000.0;
    let wall_thickness = 60.0;
    let screen_half_width = 1366.0 / 2.0;

    let wall_positions = [
        -screen_half_width + (wall_thickness / 2.0),
        screen_half_width - (wall_thickness / 2.0),
    ];
    let wall_names = ["Left Wall", "Right Wall"];

    for (&x_pos, &name) in wall_positions.iter().zip(wall_names.iter()) {
        let wall_entity = commands.spawn((
            Sprite::from_color(Color::srgba(0.8, 0.2, 0.2, 0.0), Vec2::new(wall_thickness, wall_height)),
            Transform::from_xyz(x_pos, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness / 2.0, wall_height / 2.0),
            avian2d::prelude::CollisionLayers::new(
                CollisionLayers::GROUND,
                CollisionLayers::BALL | CollisionLayers::PLAYER
            ),
            Restitution::new(0.9),
            Friction::new(0.2),
            ColliderDensity(1000.0),
            Name::new(name),
        )).id();

        println!("🧱 WALL SPAWNED: {} Entity {:?} at x={}, thickness={}, height={}",
                 name, wall_entity, x_pos, wall_thickness, wall_height);
    }
}

fn spawn_field_ceiling(commands: &mut Commands) {
    let screen_height = 768.0;
    let ceiling_y = screen_height / 2.0; 
    let ceiling_width = 1366.0; 
    
    commands.spawn((
        Transform::from_xyz(0.0, ceiling_y, 0.0),
        RigidBody::Static,
        Collider::rectangle(ceiling_width / 2.0, 10.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL 
        ),
        Restitution::new(0.7), 
        Name::new("Ceiling"),
    ));
}