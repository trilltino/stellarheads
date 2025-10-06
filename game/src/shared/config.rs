use bevy::prelude::*;

/// Game physics constants
#[derive(Resource, Debug, Clone)]
pub struct GamePhysics {
    pub gravity_scale: f32,
    pub player_mass: f32,
    pub player_restitution: f32,
    pub player_friction: f32,
    pub ball_restitution: f32,
    pub ball_friction: f32,
    pub ball_mass: f32,
    pub ball_gravity_scale: f32,
    pub ball_bounce_multiplier: f32,
    pub ball_max_speed: f32,
    pub terminal_velocity: f32,
}

impl Default for GamePhysics {
    fn default() -> Self {
        Self {
            gravity_scale: 20.0,
            player_mass: 1.0,
            player_restitution: 0.5,
            player_friction: 0.7,
            ball_restitution: 0.8,
            ball_friction: 0.05,
            ball_mass: 2.0,
            ball_gravity_scale: 12.0,
            ball_bounce_multiplier: 0.8,
            ball_max_speed: 400.0,
            terminal_velocity: -400.0,
        }
    }
}

/// Player movement constants
#[derive(Resource, Debug, Clone)]
pub struct PlayerMovement {
    pub speed: f32,
    pub jump_force: f32,
    pub air_control: f32,
    pub jump_cut_multiplier: f32,
    pub coyote_time_seconds: f32,
    pub kick_range: f32,
    pub kick_force: f32,
    pub kick_upward_boost: f32,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            speed: 300.0,
            jump_force: 200.0,
            air_control: 0.7,
            jump_cut_multiplier: 0.3,
            coyote_time_seconds: 0.2,
            kick_range: 60.0,
            kick_force: 400.0,
            kick_upward_boost: 50.0,
        }
    }
}

/// AI behavior constants
#[derive(Resource, Debug, Clone)]
pub struct AiBehavior {
    pub decision_interval_seconds: f32,
    pub speed_multiplier: f32,
    pub jump_force_multiplier: f32,
    pub kick_range: f32,
    pub kick_force: f32,
    pub kick_upward_boost: f32,
    pub chase_ball_distance: f32,
    pub jump_height_threshold: f32,
    pub jump_distance_threshold: f32,
    pub stop_at_target_distance: f32,
    pub kick_cooldown_seconds: f32,
}

impl Default for AiBehavior {
    fn default() -> Self {
        Self {
            decision_interval_seconds: 0.5,
            speed_multiplier: 0.7,
            jump_force_multiplier: 0.8,
            kick_range: 60.0,
            kick_force: 300.0,
            kick_upward_boost: 30.0,
            chase_ball_distance: 300.0,
            jump_height_threshold: 50.0,
            jump_distance_threshold: 100.0,
            stop_at_target_distance: 30.0,
            kick_cooldown_seconds: 0.3,
        }
    }
}

/// Game layout constants (field dimensions, positions)
#[derive(Resource, Debug, Clone)]
pub struct GameLayout {
    // Player sizing
    pub player_visual_size: f32,
    pub player_physics_radius: f32,

    // Ball sizing
    pub ball_visual_size: f32,
    pub ball_physics_radius: f32,

    // Field dimensions
    pub ground_level: f32,
    pub ground_height: f32,
    pub field_width: f32,
    pub screen_width: f32,
    pub screen_height: f32,

    // Walls and boundaries
    pub wall_height: f32,
    pub wall_thickness: f32,
    pub ceiling_y: f32,

    // Player positions
    pub player_separation: f32,
    pub left_player_x: f32,
    pub right_player_x: f32,

    // Goals
    pub left_goal_x: f32,
    pub right_goal_x: f32,
    pub goal_y: f32,
    pub goal_width: f32,
    pub goal_height: f32,
    pub post_thickness: f32,
    pub goal_x_offset: f32,

    // Ball spawn
    pub ball_spawn_x: f32,
    pub ball_spawn_y: f32,
}

impl Default for GameLayout {
    fn default() -> Self {
        let ground_level = -350.0;
        let player_physics_radius = 30.0;
        let screen_width = 1366.0;
        let screen_height = 768.0;
        let goal_width = 100.0;

        Self {
            // Player sizing
            player_visual_size: 80.0,
            player_physics_radius,

            // Ball sizing
            ball_visual_size: 48.0,
            ball_physics_radius: 24.0,

            // Field dimensions
            ground_level,
            ground_height: 50.0,
            field_width: 5000.0,
            screen_width,
            screen_height,

            // Walls and boundaries
            wall_height: 1000.0,
            wall_thickness: 60.0,
            ceiling_y: screen_height / 2.0,

            // Player positions
            player_separation: 350.0,
            left_player_x: -400.0,
            right_player_x: 400.0,

            // Goals
            left_goal_x: -400.0,
            right_goal_x: 400.0,
            goal_y: -250.0,
            goal_width,
            goal_height: 120.0,  // Match actual implementation
            post_thickness: 8.0,
            goal_x_offset: (screen_width / 2.0) - 50.0,

            // Ball spawn
            ball_spawn_x: 0.0,
            ball_spawn_y: -200.0,
        }
    }
}

impl GameLayout {
    /// Get spawn position for left player
    pub fn left_player_spawn(&self) -> Vec3 {
        Vec3::new(
            self.left_player_x,
            self.ground_level + 25.0 + self.player_physics_radius,
            0.0,
        )
    }

    /// Get spawn position for right player
    pub fn right_player_spawn(&self) -> Vec3 {
        Vec3::new(
            self.right_player_x,
            self.ground_level + 25.0 + self.player_physics_radius,
            0.0,
        )
    }

    /// Get ball spawn position
    pub fn ball_spawn(&self) -> Vec3 {
        Vec3::new(self.ball_spawn_x, self.ball_spawn_y, 0.0)
    }

    /// Get ground detection ray length
    pub fn ground_detection_distance(&self) -> f32 {
        25.0 // Half player height + buffer
    }

    /// Get ground top position (for goal placement)
    pub fn ground_top(&self) -> f32 {
        self.ground_level + (self.ground_height / 2.0)
    }

    /// Get left wall x position
    pub fn left_wall_x(&self) -> f32 {
        -(self.screen_width / 2.0) + (self.wall_thickness / 2.0)
    }

    /// Get right wall x position
    pub fn right_wall_x(&self) -> f32 {
        (self.screen_width / 2.0) - (self.wall_thickness / 2.0)
    }

    /// Get goal center y position
    pub fn goal_center_y(&self) -> f32 {
        self.ground_top() + (self.goal_height / 2.0)
    }
}

/// Camera configuration
#[derive(Resource, Debug, Clone)]
pub struct CameraConfig {
    pub scale: f32,
    pub bloom_threshold: f32,
    pub bloom_threshold_softness: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            scale: 1.5,
            bloom_threshold: 0.6,
            bloom_threshold_softness: 0.2,
        }
    }
}

/// Scoring configuration
#[derive(Resource, Debug, Clone)]
pub struct ScoringConfig {
    pub winning_score: i32,
    pub match_duration_seconds: f32,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            winning_score: 5,
            match_duration_seconds: 180.0, // 3 minutes
        }
    }
}

/// Background configuration
#[derive(Resource, Debug, Clone)]
pub struct BackgroundConfig {
    pub asset_path: String,
    pub z_depth: f32,
    pub scale: f32,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            asset_path: "gamescreen/gamescreen.png".to_string(),
            z_depth: -10.0,
            scale: 0.67, // Scale to match camera zoom (1/1.5 = 0.67)
        }
    }
}

/// Field physics configuration (ground, walls, goals)
#[derive(Resource, Debug, Clone)]
pub struct FieldPhysics {
    pub ground_restitution: f32,
    pub ground_friction: f32,
    pub wall_restitution: f32,
    pub wall_friction: f32,
    pub post_restitution: f32,
    pub ceiling_restitution: f32,
}

impl Default for FieldPhysics {
    fn default() -> Self {
        Self {
            ground_restitution: 0.1,
            ground_friction: 0.9,
            wall_restitution: 0.9,
            wall_friction: 0.2,
            post_restitution: 0.8,
            ceiling_restitution: 0.7,
        }
    }
}

/// Plugin to insert all game configuration resources
pub struct GameConfigPlugin;

impl Plugin for GameConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GamePhysics::default())
            .insert_resource(PlayerMovement::default())
            .insert_resource(AiBehavior::default())
            .insert_resource(GameLayout::default())
            .insert_resource(FieldPhysics::default())
            .insert_resource(CameraConfig::default())
            .insert_resource(ScoringConfig::default())
            .insert_resource(BackgroundConfig::default());
    }
}
