use avian2d::prelude::*;
use bevy::prelude::*;
use crate::shared::scoring::GoalTeam;
use crate::shared::{AppState, config::{GameLayout, FieldPhysics}};
use super::CollisionLayers;

#[derive(Component)]
pub struct Goal {
    pub team: GoalTeam,
}

pub fn setup_goals(
    mut commands: Commands,
    layout: Res<GameLayout>,
    field_physics: Res<FieldPhysics>,
) {
    let ground_top = layout.ground_top();

    spawn_soccer_goal(
        &mut commands,
        -layout.goal_x_offset,
        GoalTeam::Left,
        ground_top,
        &layout,
        &field_physics,
    );

    spawn_soccer_goal(
        &mut commands,
        layout.goal_x_offset,
        GoalTeam::Right,
        ground_top,
        &layout,
        &field_physics,
    );
}

fn spawn_soccer_goal(
    commands: &mut Commands,
    x_position: f32,
    team: GoalTeam,
    ground_top: f32,
    layout: &GameLayout,
    field_physics: &FieldPhysics,
) {
    let goal_center_y = layout.goal_center_y();

    // Invisible goal posts for collision (background shows visual goals)
    let post_half_height = layout.goal_height / 2.0;

    // Top post (invisible collision)
    commands.spawn((
        Sprite::from_color(
            Color::srgba(1.0, 1.0, 1.0, 0.0),
            Vec2::new(layout.post_thickness, post_half_height)
        ),
        Transform::from_xyz(x_position, ground_top + (layout.goal_height * 0.75), 0.0),
        RigidBody::Static,
        Collider::rectangle(layout.post_thickness / 2.0, post_half_height / 2.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Restitution::new(field_physics.post_restitution),
        Name::new("Goal Post Top"),
    ));

    // Invisible crossbar
    commands.spawn((
        Sprite::from_color(
            Color::srgba(1.0, 1.0, 1.0, 0.0),
            Vec2::new(layout.goal_width, layout.post_thickness)
        ),
        Transform::from_xyz(x_position, ground_top + layout.goal_height, 0.0),
        RigidBody::Static,
        Collider::rectangle(layout.goal_width / 2.0, layout.post_thickness / 2.0),
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GROUND,
            CollisionLayers::BALL | CollisionLayers::PLAYER
        ),
        Restitution::new(field_physics.post_restitution),
        Name::new("Goal Crossbar"),
    ));

    // Goal line sensor (spans entire goal width at the goal line)
    let goal_entity = commands.spawn((
        // Invisible goal line sensor (background shows visual goals)
        Sprite::from_color(
            Color::srgba(1.0, 0.0, 0.0, 0.0),
            Vec2::new(layout.post_thickness, layout.goal_height)
        ),
        Transform::from_xyz(x_position, goal_center_y, 0.0),
        RigidBody::Static,
        // Wide sensor covering full goal mouth
        Collider::rectangle(layout.goal_width / 2.0, layout.goal_height / 2.0),
        Sensor, // This is key - sensor doesn't physically block but detects collision
        avian2d::prelude::CollisionLayers::new(
            CollisionLayers::GOAL,
            CollisionLayers::BALL
        ),
        Goal { team: team.clone() },
        Name::new(format!("{team:?} GOAL LINE SENSOR")),
    )).id();

    #[cfg(not(target_arch = "wasm32"))]
    println!("🥅 EFFICIENT GOAL SPAWNED: {team:?} team");
    #[cfg(not(target_arch = "wasm32"))]
    println!("   📍 Goal Line Sensor: Entity {goal_entity:?} at ({x_position}, {goal_center_y})");
    #[cfg(not(target_arch = "wasm32"))]
    println!("   📐 Sensor Size: {}x{} (covers full goal mouth)", layout.goal_width, layout.goal_height);
    #[cfg(not(target_arch = "wasm32"))]
    println!("   🎯 Instant Detection: Ball crossing goal line triggers immediately");
}

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_goals);
    }
}
