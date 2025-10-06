use avian2d::prelude::*;
use bevy::prelude::*;
use crate::shared::AppState;
use crate::shared::scoring::{GoalScored, GoalTeam};
use crate::shared::audio::music_system::{PlayKickSoundEvent, PlayKickSound};
use super::{Ball, Goal, Player};


#[derive(Copy, Clone)]
pub struct CollisionLayers;

impl CollisionLayers {
    pub const BALL: u32 = 1 << 0;
    pub const GOAL: u32 = 1 << 1;
    pub const PLAYER: u32 = 1 << 2;
    pub const GROUND: u32 = 1 << 3;
}


pub fn ball_kick_collision_system(
    mut collision_events: EventReader<CollisionStarted>,
    balls: Query<Entity, With<Ball>>,
    players: Query<Entity, With<Player>>,
    mut kick_events: EventWriter<PlayKickSound>,
) {
    for collision in collision_events.read() {
        let entity_a = collision.0;
        let entity_b = collision.1;

        let is_ball_player_collision =
            (balls.contains(entity_a) && players.contains(entity_b)) ||
            (balls.contains(entity_b) && players.contains(entity_a));

        if is_ball_player_collision {
            kick_events.write(PlayKickSoundEvent);
            println!("⚽ Player kicked the ball!");
        }
    }
}


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
        println!("   Ball entities: {ball_count}, Goal entities: {goal_count}");
    }
    
    // Debug state issues
    if ball_count == 0 || goal_count == 0 {
        static mut WARNED_MISSING_ENTITIES: bool = false;
        unsafe {
            if !WARNED_MISSING_ENTITIES {
                println!("⚠️  MISSING ENTITIES: {ball_count} balls, {goal_count} goals - Make sure you're in InGame state!");
                WARNED_MISSING_ENTITIES = true;
            }
        }
    }
    
    for collision in collision_events.read() {
        let entity_a = collision.0;
        let entity_b = collision.1;

        // Get component details for debugging
        let _name_a = "Entity A";
        let _name_b = "Entity B";

        println!("🔍 COLLISION EVENT: {entity_a:?} <-> {entity_b:?}");
        println!("   A: ball={}, goal={}", balls.contains(entity_a), goals.contains(entity_a));
        println!("   B: ball={}, goal={}", balls.contains(entity_b), goals.contains(entity_b));

        // Check if either entity has the Ball component AND either has Goal component
        if balls.contains(entity_a) || balls.contains(entity_b) {
            println!("   🏀 BALL DETECTED in collision!");
        }
        if goals.contains(entity_a) || goals.contains(entity_b) {
            println!("   🥅 GOAL DETECTED in collision!");
        }

        let (ball_entity, goal_entity) = if balls.contains(entity_a) && goals.contains(entity_b) {
            println!("   ✅ BALL-GOAL MATCH: A=ball, B=goal");
            (entity_a, entity_b)
        } else if balls.contains(entity_b) && goals.contains(entity_a) {
            println!("   ✅ BALL-GOAL MATCH: B=ball, A=goal");
            (entity_b, entity_a)
        } else {
            println!("   ❌ Not a ball-goal collision");
            continue; 
        };

        if let Ok((goal, goal_transform, _)) = goals.get(goal_entity) {
            println!("GOAL COLLISION DETECTED! Ball hit {:?} goal sensor at {:?}", goal.team, goal_transform.translation);
            
            let scoring_team = match goal.team {
                GoalTeam::Left => GoalTeam::Right,   
                GoalTeam::Right => GoalTeam::Left,  
            };

            let goal_position = goal_transform.translation;

            println!("📤 WRITING GoalScored event for {scoring_team:?} team!");
            score_events.write(GoalScored {
                goal_position,
                scoring_team: scoring_team.clone(),
            });
            println!("✅ GoalScored event written successfully!");

            if let Ok(mut velocity) = ball_velocities.get_mut(ball_entity) {
                let nudge_force = 150.0;
                match goal.team {
                    GoalTeam::Left => {
                        velocity.x = velocity.x.abs().max(nudge_force);
                    }
                    GoalTeam::Right => {
                        velocity.x = -velocity.x.abs().max(nudge_force);
                    }
                }
            }

            println!(
                "⚽ GOAL SCORED! {scoring_team:?} team scored at position {goal_position:?}"
            );
        }
    }
}


// BACKUP GOAL DETECTION: Position-based system that doesn't rely on collision events
pub fn position_based_goal_detection(
    balls: Query<(Entity, &Transform), With<Ball>>,
    _goals: Query<(&Goal, &Transform), With<Goal>>,
    mut score_events: EventWriter<GoalScored>,
    mut ball_velocities: Query<&mut LinearVelocity, With<Ball>>,
    mut last_ball_positions: Local<std::collections::HashMap<Entity, Vec3>>,
) {
    let screen_width = 1366.0;
    let goal_x_threshold = (screen_width / 2.0) - 100.0; // Same as goal positions
    let ground_top = -350.0 + 25.0; // Ground level + half height
    let goal_height = 120.0;
    let goal_bottom = ground_top;
    let goal_top = ground_top + goal_height;

    for (ball_entity, ball_transform) in balls.iter() {
        let ball_pos = ball_transform.translation;
        let last_pos = last_ball_positions.get(&ball_entity).copied().unwrap_or(ball_pos);

        // Check if ball is within goal height range
        if ball_pos.y >= goal_bottom && ball_pos.y <= goal_top {
            // Check for left goal (ball crosses left goal line from right to left)
            if last_pos.x > -goal_x_threshold && ball_pos.x <= -goal_x_threshold {
                println!("🚨 POSITION-BASED GOAL DETECTION: LEFT GOAL!");
                println!("   Ball moved from ({}, {}) to ({}, {})", last_pos.x, last_pos.y, ball_pos.x, ball_pos.y);

                score_events.write(GoalScored {
                    goal_position: Vec3::new(-goal_x_threshold, ball_pos.y, 0.0),
                    scoring_team: GoalTeam::Right, // Right team scores on left goal
                });

                // Bounce ball back
                if let Ok(mut velocity) = ball_velocities.get_mut(ball_entity) {
                    velocity.x = velocity.x.abs().max(150.0);
                }
            }
            // Check for right goal (ball crosses right goal line from left to right)
            else if last_pos.x < goal_x_threshold && ball_pos.x >= goal_x_threshold {
                println!("🚨 POSITION-BASED GOAL DETECTION: RIGHT GOAL!");
                println!("   Ball moved from ({}, {}) to ({}, {})", last_pos.x, last_pos.y, ball_pos.x, ball_pos.y);

                score_events.write(GoalScored {
                    goal_position: Vec3::new(goal_x_threshold, ball_pos.y, 0.0),
                    scoring_team: GoalTeam::Left, // Left team scores on right goal
                });

                // Bounce ball back
                if let Ok(mut velocity) = ball_velocities.get_mut(ball_entity) {
                    velocity.x = -velocity.x.abs().max(150.0);
                }
            }
        }

        // Update last position
        last_ball_positions.insert(ball_entity, ball_pos);
    }
}

fn debug_collisions(
    mut collision_events: EventReader<CollisionStarted>,
    names: Query<&Name>,
    balls: Query<Entity, With<Ball>>,
    goals: Query<Entity, With<Goal>>,
) {
    let collision_count = collision_events.len();
    
    
    if collision_count > 0 {
        println!("🔔 COLLISION EVENTS: {collision_count} events detected!");
    }
    
    for collision in collision_events.read() {
        let entity_a = collision.0;
        let entity_b = collision.1;
        
        let name_a = names.get(entity_a).map(|n| n.as_str()).unwrap_or("Unknown");
        let name_b = names.get(entity_b).map(|n| n.as_str()).unwrap_or("Unknown");
        
        println!("💥 COLLISION: {name_a} <-> {name_b}");
        
        let is_ball_goal = (balls.contains(entity_a) && goals.contains(entity_b)) || 
                          (balls.contains(entity_b) && goals.contains(entity_a));
        
        if is_ball_goal {
            println!("🚨🚨🚨 BALL-GOAL COLLISION DETECTED!!! {name_a} <-> {name_b} 🚨🚨🚨");
        }
    }
}

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
                    score_on_goal_collision.run_if(in_state(AppState::InGame)),
                    position_based_goal_detection.run_if(in_state(AppState::InGame)), // BACKUP SYSTEM
                    ball_kick_collision_system.run_if(in_state(AppState::InGame)),
                    debug_collisions,
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