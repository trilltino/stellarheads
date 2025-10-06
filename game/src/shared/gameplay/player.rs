use avian2d::prelude::*;
use bevy::prelude::*;
use super::{Ball, CollisionLayers};
use crate::shared::{
    scoring::PlayerReset,
    AppState,
    config::{GamePhysics, PlayerMovement, AiBehavior as AiConfig, GameLayout},
};
use crate::shared::audio::music_system::{PlayKickSoundEvent, PlayKickSound};
use rand::seq::SliceRandom;

type PlayerMovementQuery<'a> = (
    &'a mut LinearVelocity,
    &'a mut Transform,
    &'a Speed,
    &'a JumpForce,
    &'a IsGrounded,
    &'a mut CoyoteTime,
);

type BallQuery<'a> = (&'a mut LinearVelocity, &'a Transform);
type PlayerTransformQuery<'a> = &'a Transform;

type AiMovementQuery<'a> = (
    &'a mut AiPlayer,
    &'a mut LinearVelocity,
    &'a mut Transform,
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
    pub kick_cooldown: Timer,
}

#[derive(Clone, PartialEq, Reflect)]
pub enum AiBehavior {
    ChaseBall,
    ReturnToPosition,
    DefendGoal,
}

impl AiPlayer {
    pub fn new(ai_config: &AiConfig) -> Self {
        Self {
            decision_timer: Timer::from_seconds(ai_config.decision_interval_seconds, TimerMode::Repeating),
            current_target: Vec2::ZERO,
            behavior_state: AiBehavior::ChaseBall,
            kick_cooldown: Timer::from_seconds(ai_config.kick_cooldown_seconds, TimerMode::Once),
        }
    }
}

impl Default for AiPlayer {
    fn default() -> Self {
        Self {
            decision_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            current_target: Vec2::ZERO,
            behavior_state: AiBehavior::ChaseBall,
            kick_cooldown: Timer::from_seconds(0.3, TimerMode::Once),
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

impl CoyoteTime {
    pub fn new(coyote_time_seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(coyote_time_seconds, TimerMode::Once),
            was_grounded: false,
        }
    }
}

impl Default for CoyoteTime {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Once),
            was_grounded: false,
        }
    }
}

#[derive(Component)]
#[allow(dead_code)] // Reserved for future player customization
pub struct PlayerTexture(Handle<Image>);

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
    pub fn new(
        texture: Handle<Image>,
        position: Vec3,
        physics: &GamePhysics,
        movement: &PlayerMovement,
        layout: &GameLayout,
    ) -> Self {
        Self {
            sprite: Sprite {
                image: texture,
                custom_size: Some(Vec2::new(layout.player_visual_size, layout.player_visual_size)),
                ..default()
            },
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(layout.player_physics_radius),
            restitution: Restitution::new(physics.player_restitution),
            friction: Friction::new(physics.player_friction),
            velocity: LinearVelocity::ZERO,
            gravity_scale: GravityScale(physics.gravity_scale),
            speed: Speed(movement.speed),
            jump_force: JumpForce(movement.jump_force),
            is_grounded: IsGrounded(false),
            coyote_time: CoyoteTime::new(movement.coyote_time_seconds),
            marker: Player,
            mass: Mass(physics.player_mass),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            layers: avian2d::prelude::CollisionLayers::new(
                CollisionLayers::PLAYER,
                CollisionLayers::BALL | CollisionLayers::GROUND | CollisionLayers::PLAYER
            ),
        }
    }
}

fn cleanup_players(
    mut commands: Commands,
    player_query: Query<Entity, Or<(With<LocalPlayer>, With<AiPlayer>)>>,
) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
        #[cfg(not(target_arch = "wasm32"))]
        println!("🗑️ Despawned player entity: {entity:?}");
    }
    #[cfg(not(target_arch = "wasm32"))]
    println!("🧹 All players cleaned up for new game");
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    physics: Res<GamePhysics>,
    movement: Res<PlayerMovement>,
    ai_config: Res<AiConfig>,
    layout: Res<GameLayout>,
) {
    // Available player textures
    let player_textures = ["player/Player 1.png",
        "player/Player2.png",
        "player/Player3.png",
        "player/Player4.png"];

    // Randomly select 2 different players for this game
    let mut rng = rand::thread_rng();
    let selected_players: Vec<_> = player_textures.choose_multiple(&mut rng, 2).collect();

    let local_player_texture = asset_server.load(*selected_players[0]);
    let ai_player_texture = asset_server.load(*selected_players[1]);

    #[cfg(not(target_arch = "wasm32"))]
    println!("🎲 Random player selection: Local Player={}, AI Player={}", selected_players[0], selected_players[1]);

    let left_player = commands.spawn((
        PlayerBundle::new(
            local_player_texture,
            layout.left_player_spawn(),
            &physics,
            &movement,
            &layout,
        ),
        LocalPlayer,
        Name::new("LocalPlayer"),
    )).id();

    let right_player = commands.spawn((
        PlayerBundle::new(
            ai_player_texture,
            layout.right_player_spawn(),
            &physics,
            &movement,
            &layout,
        ),
        AiPlayer::new(&ai_config),
        Name::new("AIPlayer"),
    )).id();

    #[cfg(not(target_arch = "wasm32"))]
    println!(
        "🕹️ PLAYERS SPAWNED: Left={:?} at {:?}, Right={:?} at {:?}",
        left_player, layout.left_player_spawn(),
        right_player, layout.right_player_spawn()
    );
}

fn player_movement_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    movement: Res<PlayerMovement>,
    physics: Res<GamePhysics>,
    mut q: Query<PlayerMovementQuery, (With<Player>, With<LocalPlayer>)>,
) {
    for (mut velocity, mut transform, speed, jump_force, is_grounded, mut coyote_time) in &mut q {
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

        // Horizontal movement input
        let mut x_input = 0.0;
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            x_input -= 1.0;
            transform.scale.x = -1.0; // Face left
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            x_input += 1.0;
            transform.scale.x = 1.0; // Face right
        }

        // Apply movement with air control
        if is_grounded.0 {
            velocity.x = x_input * speed.0;
        } else {
            velocity.x = velocity.x.lerp(
                x_input * speed.0,
                movement.air_control * time.delta_secs() * 10.0
            );
        }

        // Jump handling with coyote time
        let can_jump = is_grounded.0 || coyote_time.was_grounded;
        if keys.just_pressed(KeyCode::Space) && can_jump {
            velocity.y = jump_force.0;
            coyote_time.was_grounded = false;
        }

        // Variable jump height (cut jump short on release)
        if keys.just_released(KeyCode::Space) && velocity.y > 0.0 {
            velocity.y *= movement.jump_cut_multiplier;
        }

        // Apply terminal velocity
        if velocity.y < physics.terminal_velocity {
            velocity.y = physics.terminal_velocity;
        }
    }
}

fn player_ball_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    movement: Res<PlayerMovement>,
    mut ball_query: Query<BallQuery, (With<Ball>, Without<Player>)>,
    player_query: Query<PlayerTransformQuery, (With<Player>, With<LocalPlayer>)>,
    mut kick_events: EventWriter<PlayKickSound>,
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

    // Kick ball if within range
    if distance < movement.kick_range {
        let kick_direction = (ball_pos - player_pos).normalize_or_zero();

        ball_velocity.x += kick_direction.x * movement.kick_force;
        ball_velocity.y += kick_direction.y * movement.kick_force + movement.kick_upward_boost;

        kick_events.write(PlayKickSoundEvent);
        #[cfg(not(target_arch = "wasm32"))]
        println!("Player kicked the ball!");
    }
}

fn ai_ball_interaction(
    time: Res<Time>,
    ai_config: Res<AiConfig>,
    layout: Res<GameLayout>,
    mut ball_query: Query<BallQuery, (With<Ball>, Without<Player>)>,
    mut ai_query: Query<(&Transform, &mut AiPlayer), (With<Player>, With<AiPlayer>)>,
    mut kick_events: EventWriter<PlayKickSound>,
) {
    let Ok((mut ball_velocity, ball_transform)) = ball_query.single_mut() else {
        return;
    };

    for (ai_transform, mut ai_player) in ai_query.iter_mut() {
        ai_player.kick_cooldown.tick(time.delta());

        let ai_pos = ai_transform.translation.truncate();
        let ball_pos = ball_transform.translation.truncate();
        let distance = ai_pos.distance(ball_pos);

        if distance < ai_config.kick_range
            && ai_player.behavior_state == AiBehavior::ChaseBall
            && ai_player.kick_cooldown.finished()
        {
            // Kick toward left goal
            let kick_direction = (Vec2::new(layout.left_goal_x, layout.goal_y) - ball_pos)
                .normalize_or_zero();

            ball_velocity.x += kick_direction.x * ai_config.kick_force;
            ball_velocity.y += kick_direction.y * ai_config.kick_force + ai_config.kick_upward_boost;

            ai_player.kick_cooldown.reset();
            kick_events.write(PlayKickSoundEvent);

            #[cfg(not(target_arch = "wasm32"))]
            println!("AI kicked the ball!");
        }
    }
}

fn ground_detection(
    layout: Res<GameLayout>,
    mut player_query: Query<(Entity, &mut IsGrounded, &Transform), With<Player>>,
    spatial_query: SpatialQuery,
) {
    for (entity, mut is_grounded, transform) in &mut player_query {
        let ray_origin = transform.translation.truncate();
        let ray_direction = Dir2::new(Vec2::new(0.0, -1.0)).unwrap();
        let max_distance = layout.ground_detection_distance();
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);

        is_grounded.0 = spatial_query.cast_ray(ray_origin, ray_direction, max_distance, true, &filter).is_some();
    }
}

// AI Movement System
fn ai_player_movement(
    time: Res<Time>,
    ai_config: Res<AiConfig>,
    layout: Res<GameLayout>,
    ball_query: Query<&Transform, (With<Ball>, Without<Player>)>,
    mut ai_query: Query<AiMovementQuery, (With<Player>, Without<LocalPlayer>)>,
) {
    let Ok(ball_transform) = ball_query.single() else {
        return;
    };

    for (mut ai, mut velocity, mut transform, speed, jump_force, is_grounded) in ai_query.iter_mut() {
        ai.decision_timer.tick(time.delta());

        if ai.decision_timer.just_finished() {
            let ai_pos = transform.translation.truncate();
            let ball_pos = ball_transform.translation.truncate();
            let distance_to_ball = ai_pos.distance(ball_pos);

            if distance_to_ball < ai_config.chase_ball_distance {
                ai.behavior_state = AiBehavior::ChaseBall;
                ai.current_target = ball_pos;
            } else {
                ai.behavior_state = AiBehavior::ReturnToPosition;
                ai.current_target = Vec2::new(layout.right_player_x, layout.goal_y);
            }
        }

        let ai_pos = transform.translation.truncate();
        let ball_pos = ball_transform.translation.truncate();
        let direction_to_target = (ai.current_target - ai_pos).normalize_or_zero();

        // Make AI face the ball
        let direction_to_ball = (ball_pos - ai_pos).normalize_or_zero();
        if direction_to_ball.x > 0.1 {
            transform.scale.x = 1.0; // Face right
        } else if direction_to_ball.x < -0.1 {
            transform.scale.x = -1.0; // Face left
        }

        // Apply AI movement with speed multiplier
        let ai_speed = speed.0 * ai_config.speed_multiplier;
        let target_x_velocity = direction_to_target.x * ai_speed;
        velocity.x = velocity.x.lerp(target_x_velocity, time.delta_secs() * 5.0);

        // AI jumping logic
        let should_jump = is_grounded.0
            && ball_pos.y > ai_pos.y + ai_config.jump_height_threshold
            && ai_pos.distance(ball_pos) < ai_config.jump_distance_threshold;

        if should_jump && velocity.y.abs() < 10.0 {
            velocity.y = jump_force.0 * ai_config.jump_force_multiplier;
        }

        // Slow down when reaching target
        if ai_pos.distance(ai.current_target) < ai_config.stop_at_target_distance {
            velocity.x *= 0.5;
        }
    }
}

fn reset_player_positions(
    layout: Res<GameLayout>,
    mut reset_events: EventReader<PlayerReset>,
    mut player_query: Query<(&mut Transform, &mut LinearVelocity), With<Player>>,
    mut ball_query: Query<(&mut Transform, &mut LinearVelocity), (With<Ball>, Without<Player>)>,
) {
    for _ in reset_events.read() {
        for (mut transform, mut velocity) in player_query.iter_mut() {
            velocity.x = 0.0;
            velocity.y = 0.0;

            // Reset to spawn positions based on side
            if transform.translation.x < 0.0 {
                transform.translation = layout.left_player_spawn();
            } else {
                transform.translation = layout.right_player_spawn();
            }
        }

        // Reset ball
        if let Ok((mut ball_transform, mut ball_velocity)) = ball_query.single_mut() {
            ball_transform.translation = layout.ball_spawn();
            ball_velocity.x = 0.0;
            ball_velocity.y = 0.0;
        }

        #[cfg(not(target_arch = "wasm32"))]
        println!("Player positions and ball reset!");
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerReset>()
            .add_systems(OnEnter(AppState::InGame), (cleanup_players, spawn_player).chain())
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
