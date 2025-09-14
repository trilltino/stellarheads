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
    let ground_level = -350.0; 
    let ground_height = 50.0; 
    let ground_top = ground_level + (ground_height / 2.0); 

    let goal_width = 100.0;
    let goal_height = 120.0;
    let post_thickness = 8.0;
    let net_thickness = 5.0;
    let goal_depth = 40.0; 
    

    let screen_width = 1366.0;
    let goal_x_offset = (screen_width / 2.0) - 100.0; 
    spawn_soccer_goal(&mut commands, -goal_x_offset, GoalTeam::Left, ground_top, goal_width, goal_height, post_thickness, net_thickness, goal_depth);
    spawn_soccer_goal(&mut commands, goal_x_offset, GoalTeam::Right, ground_top, goal_width, goal_height, post_thickness, net_thickness, goal_depth); 
}


fn spawn_soccer_goal(
    commands: &mut Commands,
    x_position: f32,
    team: GoalTeam,
    ground_top: f32,
    goal_width: f32,
    goal_height: f32,
    post_thickness: f32,
    _net_thickness: f32,
    _goal_depth: f32,
) {
    let goal_center_y = ground_top + (goal_height / 2.0);

    // 1. VISUAL GOAL POSTS (minimal collision, mainly for visual boundaries)
    let post_half_height = goal_height / 2.0;

    // Left post (top half)
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::new(post_thickness, post_half_height)),
        Transform::from_xyz(x_position, ground_top + (goal_height * 0.75), 0.0),
        RigidBody::Static,
        Collider::rectangle(post_thickness / 2.0, post_half_height / 2.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Restitution::new(0.8),
        Name::new("Goal Post Top"),
    ));

    // Crossbar
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::new(goal_width, post_thickness)),
        Transform::from_xyz(x_position, ground_top + goal_height, 0.0),
        RigidBody::Static,
        Collider::rectangle(goal_width / 2.0, post_thickness / 2.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Restitution::new(0.8),
        Name::new("Goal Crossbar"),
    ));

    // 2. GOAL LINE SENSOR (spans entire goal width at the goal line)
    let goal_entity = commands.spawn((
        // Visual indicator (semi-transparent red line at goal line)
        Sprite::from_color(Color::srgba(1.0, 0.0, 0.0, 0.5), Vec2::new(post_thickness, goal_height)),
        Transform::from_xyz(x_position, goal_center_y, 0.0),
        RigidBody::Static,
        // Wide sensor covering full goal mouth
        Collider::rectangle(goal_width / 2.0, goal_height / 2.0),
        Sensor, // This is key - sensor doesn't physically block but detects collision
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GOAL,
            CollisionLayers::BALL
        ),
        Goal { team: team.clone() },
        Name::new(format!("{:?} GOAL LINE SENSOR", team)),
    )).id();

    println!("🥅 EFFICIENT GOAL SPAWNED: {:?} team", team);
    println!("   📍 Goal Line Sensor: Entity {:?} at ({}, {})", goal_entity, x_position, goal_center_y);
    println!("   📐 Sensor Size: {}x{} (covers full goal mouth)", goal_width, goal_height);
    println!("   🎯 Instant Detection: Ball crossing goal line triggers immediately");
}

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_goals);
    }
}