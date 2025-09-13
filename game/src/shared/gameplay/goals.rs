use avian2d::prelude::*;
use bevy::prelude::*;
use crate::shared::scoring::GoalTeam;
use super::CollisionLayers;
use crate::shared::AppState;

#[derive(Component)]
pub struct Goal {
    pub team: GoalTeam,
}

pub fn setup_goals(mut commands: Commands) {
    let ground_level = -350.0; // Match ground.rs ground position for 1366x768
    let ground_height = 50.0; // Match ground.rs ground sprite height  
    let ground_top = ground_level + (ground_height / 2.0); // -325.0
    
    // Goal net dimensions perfectly scaled for 1366x768
    let goal_width = 100.0;
    let goal_height = 120.0;
    let post_thickness = 8.0;
    let net_thickness = 5.0;
    let goal_depth = 40.0; // Slightly deeper for better detection
    
    // Position goals at exact screen boundaries for 1366x768
    let screen_width = 1366.0;
    let goal_x_offset = (screen_width / 2.0) - 100.0; // 583px from center
    
    // Left goal (facing right) - positioned precisely for 1366x768 screen
    spawn_soccer_goal(&mut commands, -goal_x_offset, GoalTeam::Left, ground_top, goal_width, goal_height, post_thickness, net_thickness, goal_depth);
    
    // Right goal (facing left) - positioned precisely for 1366x768 screen  
    spawn_soccer_goal(&mut commands, goal_x_offset, GoalTeam::Right, ground_top, goal_width, goal_height, post_thickness, net_thickness, goal_depth);
    
    println!("🥅 Goals spawned at ±{} for 1366x768 screen", goal_x_offset);
}

fn spawn_soccer_goal(
    commands: &mut Commands,
    x_position: f32,
    team: GoalTeam,
    ground_top: f32,
    _goal_width: f32,
    goal_height: f32,
    post_thickness: f32,
    net_thickness: f32,
    goal_depth: f32,
) {
    let side_multiplier = if matches!(team, GoalTeam::Left) { -1.0 } else { 1.0 }; // Left goal extends left, right goal extends right
    let goal_center_y = ground_top + (goal_height / 2.0);
    let goal_back_x = x_position + (goal_depth * side_multiplier);
    
    // SOLID goal post at goal mouth to block players - make it very visible and thick
    let post_entity = commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::new(post_thickness * 3.0, goal_height)), // Red and thicker for visibility
        Transform::from_xyz(x_position, goal_center_y, 0.0),
        RigidBody::Static,
        Collider::rectangle((post_thickness * 3.0) / 2.0, goal_height / 2.0), // Thicker collider
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND, // This entity IS ground type
            CollisionLayers::BALL | CollisionLayers::PLAYER // It COLLIDES WITH ball and player
        ),
        Restitution::new(0.8),
        Friction::new(0.3),
        Name::new("Goal Post Blocker"),
    )).id();
    
    println!("🚧 SOLID GOAL POST: {:?} team Entity {:?} at ({}, {}) - RED post blocks players!", 
             team, post_entity, x_position, goal_center_y);
    
    println!("🚧 Added solid goal post at ({}, {}) to block players", x_position, goal_center_y);
    
    // Goal crossbar (top) - blocks ball and players from going over
    let crossbar_entity = commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::new(goal_depth + 20.0, post_thickness * 2.0)), // Wider and thicker
        Transform::from_xyz(x_position + (goal_depth * side_multiplier / 2.0), ground_top + goal_height, 0.0),
        RigidBody::Static,
        Collider::rectangle((goal_depth + 20.0) / 2.0, post_thickness),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Restitution::new(0.8),
        Name::new("Goal Crossbar"),
    )).id();
    
    println!("🏒 CROSSBAR: Entity {:?} at ({}, {})", crossbar_entity, 
             x_position + (goal_depth * side_multiplier / 2.0), ground_top + goal_height);
    
    // Goal back net
    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.9, 0.9), Vec2::new(net_thickness, goal_height)),
        Transform::from_xyz(goal_back_x, goal_center_y, 0.0),
        RigidBody::Static,
        Collider::rectangle(net_thickness / 2.0, goal_height / 2.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL
        ),
        Restitution::new(0.1),
        Name::new("Goal Back Net"),
    ));
    
    // No goal floor - main continuous ground covers everything
    
    // ULTRA-MASSIVE goal sensor - covers entire goal area and extends beyond
    let sensor_width = goal_depth + 25.0; // Much bigger than before
    let sensor_height = goal_height + 40.0; // Much taller than before
    let sensor_x = x_position + (goal_depth * side_multiplier * 0.4); // Positioned inside goal
    
    let goal_entity = commands.spawn((
        Sprite::from_color(Color::srgba(1.0, 1.0, 0.0, 0.8), Vec2::new(sensor_width, sensor_height)), // Bright yellow for visibility
        Transform::from_xyz(sensor_x, goal_center_y, 0.0),
        RigidBody::Static,
        Collider::rectangle(sensor_width / 2.0, sensor_height / 2.0), // ENORMOUS sensor
        Sensor, // Critical: This makes it a trigger, not a solid collision
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GOAL, // This entity is a GOAL
            CollisionLayers::BALL  // It detects BALL entities
        ),
        Goal { team: team.clone() },
        Name::new(format!("{:?} GOAL SENSOR", team)),
    )).id();
    
    println!("🥅 GOAL SENSOR SPAWNED: {:?} team Entity {:?} at ({}, {}) with layers GOAL={} (collides with BALL={})", 
             team, goal_entity, 
             x_position + (goal_depth * side_multiplier * 0.5), goal_center_y,
             CollisionLayers::GOAL, CollisionLayers::BALL);
    
    println!("⚽ Spawned MASSIVE {:?} goal sensor at x: {}, size: {}x{}", 
        team, 
        x_position + (goal_depth * side_multiplier * 0.5), 
        goal_depth + 10.0, 
        goal_height + 20.0
    );
    
    println!("Spawned {:?} goal at x: {}, ground_top: {}", team, x_position, ground_top);
}

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        // Spawn goals when entering InGame state, same as ball
        app.add_systems(OnEnter(AppState::InGame), setup_goals);
    }
}