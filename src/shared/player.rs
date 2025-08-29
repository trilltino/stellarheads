use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct JumpForce(f32);

#[derive(Component)]
struct IsGrounded(bool);

#[derive(Component)]
struct CoyoteTime {
    timer: Timer,
    was_grounded: bool,
}

impl Default for CoyoteTime {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Once), // 100ms coyote time
            was_grounded: false,
        }
    }
}

#[derive(Bundle)]
struct PlayerBundle {
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
            restitution: Restitution::new(0.1), // Low bounce for natural feel
            friction: Friction::new(0.7),       // Ground friction
            velocity: LinearVelocity::ZERO,
            gravity_scale: GravityScale(2.0), // Stronger gravity for snappy feel
            speed: Speed(300.0),
            jump_force: JumpForce(600.0), // Higher jump force
            is_grounded: IsGrounded(false),
            coyote_time: CoyoteTime::default(),
            marker: Player,
            mass: Mass(1.0),
            locked_axes: LockedAxes::ROTATION_LOCKED, // Prevent spinning
        }
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle::new(
        20.0,
        Color::srgb(0.2, 0.7, 0.9),
        Vec3::new(-500.0, -250.0, 0.0), // Start above ground
    ));
}

fn player_movement_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut q: Query<
        (
            &mut LinearVelocity,
            &Speed,
            &JumpForce,
            &IsGrounded,
            &mut CoyoteTime,
        ),
        With<Player>,
    >,
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

        // Variable jump height - release space early for shorter jumps
        if keys.just_released(KeyCode::Space) && velocity.y > 0.0 {
            velocity.y *= 0.5; // Cut jump short
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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                ground_detection,
                player_movement_input.after(ground_detection),
            ),
        );
    }
}
