use avian2d::prelude::*;
use bevy::prelude::*;
use crate::shared::{ball::Ball, collision::CollisionLayers, scoring::PlayerReset, state::AppState};

// Type aliases for complex query types
type PlayerMovementQuery<'a> = (
    &'a mut LinearVelocity,
    &'a Speed,
    &'a JumpForce,
    &'a IsGrounded,
    &'a mut CoyoteTime,
);

type BallQuery<'a> = (&'a mut LinearVelocity, &'a Transform);
type PlayerTransformQuery<'a> = &'a Transform;

type GroundDetectionQuery<'a> = (Entity, &'a mut IsGrounded, &'a Transform);

type AiMovementQuery<'a> = (
    &'a mut AiPlayer,
    &'a mut LinearVelocity,
    &'a Transform,
    &'a Speed,
    &'a JumpForce,
    &'a IsGrounded,
);

#[derive(Component, Reflect)]
pub struct Player;

#[derive(Component, Reflect)]
pub struct AiPlayer {
    pub decision_timer: Timer,
    pub current_target: Vec2,
    pub behavior_state: AiBehavior,
}

#[derive(Clone, PartialEq, Reflect)]
pub enum AiBehavior {
    ChaseBall,
    ReturnToPosition,
    DefendGoal,
}

impl Default for AiPlayer {
    fn default() -> Self {
        Self {
            decision_timer: Timer::from_seconds(0.5, TimerMode::Repeating), // Make decisions every 0.5 seconds
            current_target: Vec2::ZERO,
            behavior_state: AiBehavior::ChaseBall,
        }
    }
}

#[derive(Component, Reflect)]
pub struct LocalPlayer;

#[derive(Component, Reflect)]
pub struct Speed(pub f32);

#[derive(Component, Reflect)]
pub struct JumpForce(pub f32);

#[derive(Component, Reflect)]
pub struct IsGrounded(pub bool);

#[derive(Component, Reflect)]
pub struct CoyoteTime {
    pub timer: Timer,
    pub was_grounded: bool,
}

impl Default for CoyoteTime {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Once), // 100ms coyote time
            was_grounded: false,
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    marker: Player,
    sprite: Sprite,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    restitution: Restitution,
    friction: Friction,
    velocity: LinearVelocity,
    gravity_scale: GravityScale,
    speed: Speed,
    mass: Mass,
    jump_force: JumpForce,
    is_grounded: IsGrounded,
    coyote_time: CoyoteTime,
    locked_axes: LockedAxes,
    layers: avian2d::prelude::CollisionLayers,
}

impl PlayerBundle {
    pub fn new(radius: f32, color: Color, position: Vec3) -> Self {
        Self {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(radius),
            restitution: Restitution::new(0.5), // Low bounce for natural feel
            friction: Friction::new(0.7),       // Ground friction
            velocity: LinearVelocity::ZERO,
            gravity_scale: GravityScale(20.0), // Stronger gravity for snappy feel
            speed: Speed(300.0),
            jump_force: JumpForce(200.0), // Higher jump force
            is_grounded: IsGrounded(false),
            coyote_time: CoyoteTime::default(),
            marker: Player,
            mass: Mass(1.0),
            locked_axes: LockedAxes::ROTATION_LOCKED, // Prevent spinning
            layers: avian2d::prelude::CollisionLayers::new(
                CollisionLayers::PLAYER,
                CollisionLayers::BALL | CollisionLayers::GROUND | CollisionLayers::PLAYER
            ),
        }
    }
}

fn spawn_player(mut commands: Commands) {
    // Player positions perfectly scaled for 1366x768 screen
    let player_radius = 20.0;
    let ground_level = -350.0;  // Match ground.rs
    let player_y = ground_level + 25.0 + player_radius; // Safely above ground surface
    let player_separation = 300.0; // Distance from center
    
    // Spawn local player (left side)
    let left_player = commands.spawn((
        PlayerBundle::new(
            player_radius,
            Color::srgb(0.2, 0.7, 0.9), // Blue player
            Vec3::new(-player_separation, player_y, 0.0),
        ),
        LocalPlayer,
        Name::new("LocalPlayer"),
    )).id();
    
    // Spawn AI player (right side)
    let right_player = commands.spawn((
        PlayerBundle::new(
            player_radius,
            Color::srgb(1.0, 0.4, 0.2), // Orange player
            Vec3::new(player_separation, player_y, 0.0),
        ),
        AiPlayer::default(),
        Name::new("AIPlayer"),
    )).id();
    
    println!("🕹️ PLAYERS SPAWNED: Left={:?} at ({}, {}), Right={:?} at ({}, {})", 
             left_player, -player_separation, player_y,
             right_player, player_separation, player_y);
}

fn player_movement_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut q: Query<PlayerMovementQuery, (With<Player>, With<LocalPlayer>)>,
) {
    for (mut velocity, speed, jump_force, is_grounded, mut coyote_time) in &mut q {
        // Update coyote time
        if is_grounded.0 {
            coyote_time.was_grounded = true;
            coyote_time.timer.reset();
        } else if coyote_time.was_grounded {
            coyote_time.timer.tick(time.delta());
            if coyote_time.timer.finished() {
                coyote_time.was_grounded = false;
            }
        }

        // Horizontal movement
        let mut x_input = 0.0;
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            x_input -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            x_input += 1.0;
        }

        // Apply horizontal movement with some air control
        if is_grounded.0 {
            // Full control on ground
            velocity.x = x_input * speed.0;
        } else {
            // Reduced air control for more realistic feel
            let air_control = 0.7;
            velocity.x = velocity
                .x
                .lerp(x_input * speed.0, air_control * time.delta_secs() * 10.0);
        }

        // Jump with coyote time and jump buffering
        let can_jump = is_grounded.0 || coyote_time.was_grounded;
        if keys.just_pressed(KeyCode::Space) && can_jump {
            velocity.y = jump_force.0;
            coyote_time.was_grounded = false; // Use up coyote time
        }

        // Variable jump height - more gradual fall when releasing space
        if keys.just_released(KeyCode::Space) && velocity.y > 0.0 {
            velocity.y *= 0.3; // More dramatic cut for quicker fall
        }
        
        // Add terminal velocity for more realistic falling
        const TERMINAL_VELOCITY: f32 = -400.0;
        if velocity.y < TERMINAL_VELOCITY {
            velocity.y = TERMINAL_VELOCITY;
        }
    }
}

// Ball kicking system for local player
fn player_ball_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    mut ball_query: Query<BallQuery, (With<Ball>, Without<Player>)>,
    player_query: Query<PlayerTransformQuery, (With<Player>, With<LocalPlayer>)>,
) {
    if !keys.just_pressed(KeyCode::KeyX) {
        return;
    }
    
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    
    let Ok((mut ball_velocity, ball_transform)) = ball_query.single_mut() else {
        return;
    };
    
    let player_pos = player_transform.translation.truncate();
    let ball_pos = ball_transform.translation.truncate();
    let distance = player_pos.distance(ball_pos);
    
    // Kick ball if close enough
    if distance < 60.0 {
        let kick_direction = (ball_pos - player_pos).normalize_or_zero();
        let kick_force = 400.0;
        
        ball_velocity.x += kick_direction.x * kick_force;
        ball_velocity.y += kick_direction.y * kick_force + 50.0; // Add slight upward force
        
        println!("Player kicked the ball!");
    }
}

// Ball kicking system for AI player
fn ai_ball_interaction(
    _time: Res<Time>,
    mut ball_query: Query<BallQuery, (With<Ball>, Without<Player>)>,
    mut ai_query: Query<(&Transform, &mut AiPlayer), (With<Player>, With<AiPlayer>)>,
) {
    let Ok((mut ball_velocity, ball_transform)) = ball_query.single_mut() else {
        return;
    };
    
    for (ai_transform, mut ai_player) in ai_query.iter_mut() {
        let ai_pos = ai_transform.translation.truncate();
        let ball_pos = ball_transform.translation.truncate();
        let distance = ai_pos.distance(ball_pos);
        
        // AI tries to kick when close to ball and in chase mode
        if distance < 60.0 && ai_player.behavior_state == AiBehavior::ChaseBall {
            // Add a small delay so AI doesn't kick continuously
            if ai_player.decision_timer.elapsed_secs() > 0.3 {
                let kick_direction = (Vec2::new(-400.0, -250.0) - ball_pos).normalize_or_zero(); // Kick toward left goal
                let kick_force = 300.0; // AI kicks slightly weaker than player
                
                ball_velocity.x += kick_direction.x * kick_force;
                ball_velocity.y += kick_direction.y * kick_force + 30.0; // Less upward force than player
                
                ai_player.decision_timer.reset(); // Reset timer to prevent immediate re-kick
                println!("AI kicked the ball!");
            }
        }
    }
}

fn ground_detection(
    mut player_query: Query<(Entity, &mut IsGrounded, &Transform), With<Player>>,
    spatial_query: SpatialQuery,
) {
    for (entity, mut is_grounded, transform) in &mut player_query {
        let ray_origin = transform.translation.truncate();
        let ray_direction = Dir2::new(Vec2::new(0.0, -1.0)).unwrap();
        let max_distance = 25.0; // Half player height + buffer

        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

        if let Some(_hit) =
            spatial_query.cast_ray(ray_origin, ray_direction, max_distance, true, &filter)
        {
            is_grounded.0 = true;
        } else {
            is_grounded.0 = false;
        }
    }
}

// AI Movement System
fn ai_player_movement(
    time: Res<Time>,
    ball_query: Query<&Transform, (With<Ball>, Without<Player>)>,
    mut ai_query: Query<AiMovementQuery, (With<Player>, Without<LocalPlayer>)>,
) {
    let Ok(ball_transform) = ball_query.single() else {
        return;
    };

    for (mut ai, mut velocity, transform, speed, jump_force, is_grounded) in ai_query.iter_mut() {
        // Update decision timer
        ai.decision_timer.tick(time.delta());

        if ai.decision_timer.just_finished() {
            // Make new decision every 0.5 seconds
            let ai_pos = transform.translation.truncate();
            let ball_pos = ball_transform.translation.truncate();
            let distance_to_ball = ai_pos.distance(ball_pos);
            
            // Simple AI behavior: chase ball if it's close, return to position if far
            if distance_to_ball < 300.0 {
                ai.behavior_state = AiBehavior::ChaseBall;
                ai.current_target = ball_pos;
            } else {
                ai.behavior_state = AiBehavior::ReturnToPosition;
                ai.current_target = Vec2::new(400.0, -250.0); // Home position
            }
        }

        // Execute current behavior
        let ai_pos = transform.translation.truncate();
        let direction_to_target = (ai.current_target - ai_pos).normalize_or_zero();
        
        // Horizontal movement (slower than human player)
        let ai_speed = speed.0 * 0.7; // AI moves at 70% of player speed
        let target_x_velocity = direction_to_target.x * ai_speed;
        
        // Smooth movement transition
        velocity.x = velocity.x.lerp(target_x_velocity, time.delta_secs() * 5.0);
        
        // Jump logic - jump if ball is above and AI is on ground
        let ball_pos = ball_transform.translation.truncate();
        let should_jump = is_grounded.0 
            && ball_pos.y > ai_pos.y + 50.0 
            && ai_pos.distance(ball_pos) < 100.0;
            
        if should_jump && velocity.y.abs() < 10.0 {
            velocity.y = jump_force.0 * 0.8; // AI jumps with less force
        }
        
        // Simple obstacle avoidance - stop if too close to target
        if ai_pos.distance(ai.current_target) < 30.0 {
            velocity.x *= 0.5; // Slow down when close to target
        }
    }
}

// System to reset player and ball positions after goals
fn reset_player_positions(
    mut reset_events: EventReader<PlayerReset>,
    mut player_query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
    mut ball_query: Query<(&mut Transform, &mut LinearVelocity), (With<Ball>, Without<Player>)>,
) {
    for _ in reset_events.read() {
        // Reset players to starting positions
        for (mut transform, mut velocity) in player_query.iter_mut() {
            // Stop player movement
            velocity.x = 0.0;
            velocity.y = 0.0;
            
            // Check if this is left or right player based on current x position
            if transform.translation.x < 0.0 {
                // Left player
                transform.translation = Vec3::new(-400.0, -250.0, 0.0);
            } else {
                // Right player  
                transform.translation = Vec3::new(400.0, -250.0, 0.0);
            }
        }
        
        // Reset ball to center and let it drop
        if let Ok((mut ball_transform, mut ball_velocity)) = ball_query.single_mut() {
            ball_transform.translation = Vec3::new(0.0, -200.0, 0.0); // Lower starting height
            ball_velocity.x = 0.0;
            ball_velocity.y = 0.0; // Let gravity handle the drop
        }
        
        println!("Player positions and ball reset!");
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerReset>()
            .add_systems(OnEnter(AppState::InGame), spawn_player)
            .add_systems(
                Update,
                (
                    ground_detection,
                    player_movement_input.after(ground_detection),
                    ai_player_movement.after(ground_detection),
                    player_ball_interaction,
                    ai_ball_interaction,
                    reset_player_positions,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
