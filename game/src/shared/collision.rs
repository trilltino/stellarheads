use avian2d::prelude::*;
use bevy::prelude::*;
use crate::shared::state::AppState;
use crate::shared::scoring::{GoalScored, GoalTeam};
use crate::shared::ball::Ball;
use crate::shared::goals::Goal;

// ================= Goal Detection Components =================
// Goal components are now defined in goals.rs

// ================= Collision Layers =================

#[derive(Copy, Clone)]
pub struct CollisionLayers;

impl CollisionLayers {
    pub const BALL: u32 = 1 << 0;
    pub const GOAL: u32 = 1 << 1;
    pub const PLAYER: u32 = 1 << 2;
    pub const GROUND: u32 = 1 << 3;
}

// ================= Goal Detection System =================

pub fn score_on_goal_collision(
    mut collision_events: EventReader<CollisionStarted>,
    balls: Query<Entity, With<Ball>>,
    goals: Query<(&Goal, &Transform, Entity), With<Goal>>,
    mut score_events: EventWriter<GoalScored>,
    mut ball_velocities: Query<&mut LinearVelocity, With<Ball>>,
    app_state: Res<State<AppState>>,
) {
    let collision_count = collision_events.len();
    let ball_count = balls.iter().count();
    let goal_count = goals.iter().count();
    
    if collision_count > 0 {
        println!("🔍 Checking {} collision events for goal detection... (Current state: {:?})", collision_count, app_state.get());
        println!("   Ball entities: {}, Goal entities: {}", ball_count, goal_count);
    }
    
    // Debug state issues
    if ball_count == 0 || goal_count == 0 {
        static mut WARNED_MISSING_ENTITIES: bool = false;
        unsafe {
            if !WARNED_MISSING_ENTITIES {
                println!("⚠️  MISSING ENTITIES: {} balls, {} goals - Make sure you're in InGame state!", ball_count, goal_count);
                WARNED_MISSING_ENTITIES = true;
            }
        }
    }
    
    for collision in collision_events.read() {
        let entity_a = collision.0;
        let entity_b = collision.1;
        
        println!("🔍 Collision: {:?} <-> {:?}", entity_a, entity_b);
        println!("   A: ball={}, goal={}", balls.contains(entity_a), goals.contains(entity_a));
        println!("   B: ball={}, goal={}", balls.contains(entity_b), goals.contains(entity_b));

        // Determine which entity is the ball and which is the goal
        let (ball_entity, goal_entity) = if balls.contains(entity_a) && goals.contains(entity_b) {
            println!("   ✅ BALL-GOAL MATCH: A=ball, B=goal");
            (entity_a, entity_b)
        } else if balls.contains(entity_b) && goals.contains(entity_a) {
            println!("   ✅ BALL-GOAL MATCH: B=ball, A=goal");
            (entity_b, entity_a)
        } else {
            println!("   ❌ Not a ball-goal collision");
            continue; // Not a ball-goal collision
        };

        // Process the goal collision
        if let Ok((goal, goal_transform, _)) = goals.get(goal_entity) {
            println!("GOAL COLLISION DETECTED! Ball hit {:?} goal sensor at {:?}", goal.team, goal_transform.translation);
            
            // The team that scores is opposite to the goal's team
            let scoring_team = match goal.team {
                GoalTeam::Left => GoalTeam::Right,   // Right team scores on left goal
                GoalTeam::Right => GoalTeam::Left,   // Left team scores on right goal
            };

            let goal_position = goal_transform.translation;

            // Send the goal scored event
            println!("📤 WRITING GoalScored event for {:?} team!", scoring_team);
            score_events.write(GoalScored {
                goal_position,
                scoring_team: scoring_team.clone(),
            });
            println!("✅ GoalScored event written successfully!");

            // Apply a velocity nudge to prevent ball from getting stuck
            if let Ok(mut velocity) = ball_velocities.get_mut(ball_entity) {
                let nudge_force = 150.0;
                match goal.team {
                    GoalTeam::Left => {
                        // Push ball to the right (away from left goal)
                        velocity.x = velocity.x.abs().max(nudge_force);
                    }
                    GoalTeam::Right => {
                        // Push ball to the left (away from right goal)
                        velocity.x = -velocity.x.abs().max(nudge_force);
                    }
                }
            }

            println!(
                "⚽ GOAL SCORED! {:?} team scored at position {:?}",
                scoring_team, goal_position
            );
        }
    }
}

// ================= Goal Sensor Spawning =================

// Goal sensors are now spawned directly in goals.rs as part of each goal

// Dead code removed - was not being used

// ================= Collision Debug System =================

fn debug_collisions(
    mut collision_events: EventReader<CollisionStarted>,
    names: Query<&Name>,
    balls: Query<Entity, With<Ball>>,
    goals: Query<Entity, With<Goal>>,
) {
    let collision_count = collision_events.len();
    
    // Log every frame to see if collision system is running at all
    static mut FRAME_COUNT: u32 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT % 60 == 0 { // Every second at 60fps
            println!("📊 DEBUG: Frame {}, {} collisions this frame, {} balls, {} goals", 
                     FRAME_COUNT, collision_count, balls.iter().count(), goals.iter().count());
        }
    }
    
    if collision_count > 0 {
        println!("🔔 COLLISION EVENTS: {} events detected!", collision_count);
    }
    
    for collision in collision_events.read() {
        let entity_a = collision.0;
        let entity_b = collision.1;
        
        let name_a = names.get(entity_a).map(|n| n.as_str()).unwrap_or("Unknown");
        let name_b = names.get(entity_b).map(|n| n.as_str()).unwrap_or("Unknown");
        
        println!("💥 COLLISION: {} <-> {}", name_a, name_b);
        
        // Special logging for ball-goal collisions
        let is_ball_goal = (balls.contains(entity_a) && goals.contains(entity_b)) || 
                          (balls.contains(entity_b) && goals.contains(entity_a));
        
        if is_ball_goal {
            println!("🚨🚨🚨 BALL-GOAL COLLISION DETECTED!!! {} <-> {} 🚨🚨🚨", name_a, name_b);
        }
    }
}

// ================= Collision Plugin =================

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        println!("🔧 COLLISION PLUGIN: Adding goal scoring systems...");
        println!("   - Collision layers: BALL={}, GOAL={}, PLAYER={}, GROUND={}", 
                 CollisionLayers::BALL, CollisionLayers::GOAL, CollisionLayers::PLAYER, CollisionLayers::GROUND);
        
        app.add_event::<GoalScored>()
            .add_systems(Startup, log_collision_system_startup)
            .add_systems(
                Update,
                (
                    score_on_goal_collision.run_if(in_state(AppState::InGame)), // Only in InGame state
                    debug_collisions, // Debug runs always
                ),
            );
    }
}

fn log_collision_system_startup(
    balls: Query<Entity, With<Ball>>,
    goals: Query<Entity, With<Goal>>,
) {
    println!("🚀 COLLISION SYSTEM: Starting up goal detection...");
    println!("   Found {} ball entities and {} goal entities on startup", 
             balls.iter().count(), goals.iter().count());
}