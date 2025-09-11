use crate::shared::goals::{line_segments_intersect, GoalLine, GoalScored, GoalTeam};
use crate::shared::state::AppState;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*; // Import from goal module

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

#[derive(Component)]
pub struct BallTracker {
    pub previous_position: Vec2,
    pub crossed_goals: Vec<GoalTeam>, // Track which goals have been crossed to prevent double scoring
}

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
    gravity_scale: GravityScale,
    velocity: LinearVelocity,
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
            material: MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(radius),
            restitution: Restitution::new(0.95),
            friction: Friction::new(0.05),
            mass: Mass(8.0),
            gravity_scale: GravityScale(8.0),
            velocity: LinearVelocity::ZERO,
            collider_density: ColliderDensity(1.0),
            ball: Ball {
                bounce_multiplier: 1.0,
                max_speed: 500.0,
                mass: 8.0,
                gravity_scale: 8.0,
                restitution: 1.5,
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
    commands.spawn((
        BallBundle::new(
            25.0,
            Color::srgb(1.0, 0.0, 0.0),
            Vec3::new(0.0, 200.0, 0.0),
            &mut meshes,
            &mut materials,
        ),
        Name::new("Player Ball"),
    ));
}

// Initialize ball tracker when ball is spawned
pub fn initialize_ball_tracker(
    mut commands: Commands,
    ball_query: Query<(Entity, &Transform), (With<Ball>, Without<BallTracker>)>,
) {
    for (entity, transform) in ball_query.iter() {
        commands.entity(entity).insert(BallTracker {
            previous_position: transform.translation.truncate(),
            crossed_goals: Vec::new(),
        });
    }
}

pub fn track_ball_system(
    mut ball_query: Query<(&Transform, &mut BallTracker), (With<Ball>, Changed<Transform>)>,
) {
    // This system is no longer needed - position tracking is handled in detect_goal_crossings
}

// Check for goal line crossings
pub fn detect_goal_crossings(
    mut ball_query: Query<(&Transform, &mut BallTracker), With<Ball>>,
    goal_line_query: Query<&GoalLine>,
    mut score_events: EventWriter<GoalScored>,
) {
    for (transform, mut tracker) in ball_query.iter_mut() {
        let current_pos = transform.translation.truncate();
        let previous_pos = tracker.previous_position;

        // Check if ball crossed any goal line
        for goal_line in goal_line_query.iter() {
            if line_segments_intersect(previous_pos, current_pos, goal_line.start, goal_line.end) {
                // Check if this goal hasn't been scored on recently
                if !tracker.crossed_goals.contains(&goal_line.team) {
                    // When ball crosses left goal line, right team scores (and vice versa)
                    let scoring_team = match goal_line.team {
                        GoalTeam::Left => GoalTeam::Right,
                        GoalTeam::Right => GoalTeam::Left,
                    };
                    score_events.write(GoalScored {
                        goal_position: match goal_line.team {
                            GoalTeam::Left => Vec3::new(-525.0, 0.0, 0.0),
                            GoalTeam::Right => Vec3::new(525.0, 0.0, 0.0),
                        },
                        scoring_team,
                    });
                    tracker.crossed_goals.push(goal_line.team.clone());
                }
            }
        }

        // Reset crossed goals when ball is far from goal lines
        if current_pos.x.abs() < 400.0 {
            tracker.crossed_goals.clear();
        }

        tracker.previous_position = current_pos;
    }
}

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Ball>()
            .add_systems(Startup, spawn_ball)
            .add_systems(
                Update,
                (
                    initialize_ball_tracker,
                    detect_goal_crossings,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

