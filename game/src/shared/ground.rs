use avian2d::prelude::*;
use bevy::prelude::*;
use crate::shared::collision::CollisionLayers;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ground);
    }
}

fn spawn_ground(mut commands: Commands) {
    // Create one continuous field that extends well beyond the goals
    spawn_continuous_field(&mut commands);
    
    // Create boundary walls far from the goals
    spawn_field_walls(&mut commands);
    
    // Create ceiling
    spawn_field_ceiling(&mut commands);
    
    println!("Continuous ground system spawned");
}

fn spawn_continuous_field(commands: &mut Commands) {
    // ENORMOUS platform extending far beyond goals - absolutely no gaps or falling
    let field_width = 5000.0; // Extremely wide to prevent any edge falling
    let field_height = 50.0;
    let ground_y = -350.0; // Positioned for 1366x768 screen
    
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.8, 0.2), Vec2::new(field_width, field_height)),
        Transform::from_xyz(0.0, ground_y, 0.0),
        RigidBody::Static,
        Collider::rectangle(field_width / 2.0, field_height / 2.0), // Collider matches sprite exactly
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Restitution::new(0.1), // Low bounce for realistic ground
        Friction::new(0.9),    // High friction for player control
        ColliderDensity(1000.0),
        Name::new("Continuous Field"),
    ));
    
    println!("🏟️ Spawned {}px wide continuous field at y={} - no falling possible", field_width, ground_y);
}

fn spawn_field_walls(commands: &mut Commands) {
    let wall_height = 1000.0;
    let wall_thickness = 60.0;
    // Walls positioned at exact screen edges for 1366x768 resolution
    let screen_half_width = 1366.0 / 2.0; // 683px
    let wall_positions = [-screen_half_width, screen_half_width];
    let wall_names = ["Left Wall", "Right Wall"];
    
    for (&x_pos, &name) in wall_positions.iter().zip(wall_names.iter()) {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.4, 0.4, 0.4), Vec2::new(wall_thickness, wall_height)),
            Transform::from_xyz(x_pos, 0.0, 0.0), // Centered vertically
            RigidBody::Static,
            Collider::rectangle(wall_thickness / 2.0, wall_height / 2.0),
            avian2d::prelude::CollisionLayers::new(
                CollisionLayers::GROUND,
                CollisionLayers::BALL | CollisionLayers::PLAYER
            ),
            Restitution::new(0.9), // High bounce for ball
            Friction::new(0.2),    // Low friction on walls
            Name::new(name),
        ));
        
        println!("🧱 Spawned {} at x={} for 1366x768 boundaries", name, x_pos);
    }
}

fn spawn_field_ceiling(commands: &mut Commands) {
    // Invisible ceiling to contain the ball at exact screen top for 1366x768
    let screen_height = 768.0;
    let ceiling_y = screen_height / 2.0; // +384px from center
    let ceiling_width = 1366.0; // Full screen width coverage
    
    commands.spawn((
        Transform::from_xyz(0.0, ceiling_y, 0.0),
        RigidBody::Static,
        Collider::rectangle(ceiling_width / 2.0, 10.0), // Thin but wide ceiling
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL // Only ball bounces off ceiling
        ),
        Restitution::new(0.7), // Moderate bounce
        Name::new("Ceiling"),
    ));
    
    println!("🏠 Spawned {}px wide ceiling at y={} for 1366x768 screen", ceiling_width, ceiling_y);
}