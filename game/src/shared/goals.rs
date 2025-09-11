use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::shared::state::AppState;

#[derive(Resource, Default)]
pub struct Score {
    pub left_team: u32,
    pub right_team: u32,
    pub high_score: u32,
}

#[derive(Resource)]
pub struct GameTimer {
    pub remaining_time: f32,
    pub match_duration: f32,
    pub is_finished: bool,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            remaining_time: 120.0, // 2 minutes
            match_duration: 120.0,
            is_finished: false,
        }
    }
}

#[derive(Event)]
pub struct MatchFinished {
    pub winner: Option<GoalTeam>,
}

#[derive(Event)]
pub struct PlayerReset;

impl Score {
    pub fn new() -> Self {
        Self {
            left_team: 0,
            right_team: 0,
            high_score: 0,
        }
    }

    pub fn add_point(&mut self, team: GoalTeam) {
        match team {
            GoalTeam::Left => self.left_team += 1,
            GoalTeam::Right => self.right_team += 1,
        }
        let total = self.left_team + self.right_team;
        if total > self.high_score {
            self.high_score = total;
        }
    }

    pub fn reset(&mut self) {
        self.left_team = 0;
        self.right_team = 0;
    }

    pub fn get_winner(&self) -> Option<GoalTeam> {
        if self.left_team >= 5 {
            Some(GoalTeam::Left)
        } else if self.right_team >= 5 {
            Some(GoalTeam::Right)
        } else {
            None
        }
    }

    pub fn is_match_point(&self) -> bool {
        self.left_team >= 4 || self.right_team >= 4
    }
}

// Score notifications for floating text
#[derive(Resource, Default)]
pub struct ScoreNotifications {
    pub notifications: Vec<ScoreNotification>,
}

#[derive(Clone)]
pub struct ScoreNotification {
    pub text: String,
    pub timer: f32,
    pub max_time: f32,
}

impl ScoreNotification {
    pub fn new(text: String, duration: f32) -> Self {
        Self {
            text,
            timer: duration,
            max_time: duration,
        }
    }
}

// Your existing code (events, components, etc.)
#[derive(Event)]
pub struct GoalScored {
    pub goal_position: Vec3,
    pub scoring_team: GoalTeam,
}

#[derive(Component)]
pub struct Goal {
    pub team: GoalTeam,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum GoalTeam {
    Left,
    Right,
}

#[derive(Component)]
pub struct GoalLine {
    pub start: Vec2,
    pub end: Vec2,
    pub team: GoalTeam,
}

#[derive(Bundle)]
pub struct GoalBundle {
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    goal: Goal,
}

impl GoalBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        radius: f32,
        color: Color,
        position: Vec3,
        team: GoalTeam,
    ) -> Self {
        Self {
            mesh: meshes.add(Circle::new(radius)).into(),
            material: materials.add(color).into(),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Static,
            collider: Collider::circle(radius),
            goal: Goal { team },
        }
    }
}

struct GroundInfo {
    y: f32,
    width: f32,
    height: f32,
}

impl GroundInfo {
    fn top(&self) -> f32 {
        self.y + self.height / 2.0
    }

    fn bottom(&self) -> f32 {
        self.y - self.height / 2.0
    }
}

pub fn setup_goal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let ground = GroundInfo {
        width: 1500.0,
        height: 50.0,
        y: -350.0,
    };

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.3, 0.3, 0.3),
            Vec2::new(ground.width, ground.height),
        ),
        Transform::from_xyz(0.0, ground.y, 0.0),
        RigidBody::Static,
        Collider::rectangle(ground.width, ground.height),
    ));

    let goal_radius = 20.0;
    let goal_height = 1000.0; 
    let goal_positions = [
        (
            Vec3::new(-525.0, ground.top() + goal_radius, 0.0),
            GoalTeam::Left,
        ),
        (
            Vec3::new(525.0, ground.top() + goal_radius, 0.0),
            GoalTeam::Right,
        ),
    ];

    for (position, team) in goal_positions {
        commands.spawn(GoalBundle::new(
            &mut meshes,
            &mut materials,
            goal_radius,
            Color::srgb(1.0, 0.8, 0.0),
            position,
            team,
        ));
    }

    let goal_line_x_left = -525.0;
    let goal_line_x_right = 525.0;
    let line_bottom = ground.top();
    let line_top = ground.top() + goal_height;

    commands.spawn(GoalLine {
        start: Vec2::new(goal_line_x_left, line_bottom),
        end: Vec2::new(goal_line_x_left, line_top),
        team: GoalTeam::Left,
    });

    commands.spawn(GoalLine {
        start: Vec2::new(goal_line_x_right, line_bottom),
        end: Vec2::new(goal_line_x_right, line_top),
        team: GoalTeam::Right,
    });
}

pub fn score_ui_system(mut contexts: EguiContexts, score: Res<Score>, timer: Res<GameTimer>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return; 
    };
    
    let screen_rect = ctx.screen_rect();
    let center_x = screen_rect.width() / 2.0;
    
    // Main score display (centered at top)
    egui::Window::new("Score")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(egui::pos2(center_x - 150.0, 20.0))
        .fixed_size(egui::vec2(300.0, 120.0))
        .frame(egui::Frame {
            fill: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
            corner_radius: egui::CornerRadius::same(15),
            inner_margin: egui::Margin::same(20),
            outer_margin: egui::Margin::ZERO,
            stroke: egui::Stroke::new(3.0, egui::Color32::GOLD),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("⚽ STELLAR HEADS")
                        .size(16.0)
                        .color(egui::Color32::GOLD)
                        .strong(),
                );

                ui.add_space(10.0);

                // Score display
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", score.left_team))
                                .size(48.0)
                                .color(egui::Color32::LIGHT_BLUE)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(" - ")
                                .size(36.0)
                                .color(egui::Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new(format!("{}", score.right_team))
                                .size(48.0)
                                .color(egui::Color32::LIGHT_RED)
                                .strong(),
                        );
                    });
                });

                ui.add_space(5.0);
                
                // Timer display
                let minutes = (timer.remaining_time as i32) / 60;
                let seconds = (timer.remaining_time as i32) % 60;
                let timer_color = if timer.remaining_time < 30.0 {
                    egui::Color32::LIGHT_RED
                } else {
                    egui::Color32::WHITE
                };
                
                ui.label(
                    egui::RichText::new(format!("⏰ {}:{:02}", minutes, seconds))
                        .size(20.0)
                        .color(timer_color)
                        .strong(),
                );
            });
        });
        
    // Team labels
    egui::Window::new("LeftTeam")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(egui::pos2(50.0, 60.0))
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("LEFT TEAM")
                    .size(14.0)
                    .color(egui::Color32::LIGHT_BLUE)
                    .strong(),
            );
        });
        
    egui::Window::new("RightTeam")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(egui::pos2(screen_rect.width() - 120.0, 60.0))
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("RIGHT TEAM")
                    .size(14.0)
                    .color(egui::Color32::LIGHT_RED)
                    .strong(),
            );
        });
}


pub fn score_notifications_system(
    mut contexts: EguiContexts,
    mut notifications: ResMut<ScoreNotifications>,
    time: Res<Time>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return; // Skip if context not ready
    };
    let screen_rect = ctx.screen_rect();

    notifications.notifications.retain_mut(|notif| {
        notif.timer -= time.delta_secs();
        notif.timer > 0.0
    });


    for (i, notif) in notifications.notifications.iter().enumerate() {
        let alpha = (notif.timer / notif.max_time * 255.0) as u8;
        let y_offset = 150.0 + (i as f32 * 50.0);

        egui::Window::new(format!("notification_{}", i))
            .title_bar(false)
            .resizable(false)
            .fixed_pos(egui::pos2(screen_rect.width() / 2.0 - 100.0, y_offset))
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(&notif.text)
                        .size(36.0)
                        .color(egui::Color32::from_rgba_unmultiplied(255, 215, 0, alpha))
                        .strong(),
                );
            });
    }
}

pub fn handle_goal_scored(
    mut score: ResMut<Score>,
    mut goal_events: EventReader<GoalScored>,
    mut notifications: ResMut<ScoreNotifications>,
    mut reset_events: EventWriter<PlayerReset>,
    mut match_events: EventWriter<MatchFinished>,
) {
    for event in goal_events.read() {
        score.add_point(event.scoring_team.clone());
        println!("Goal scored by {:?}! Score: {} - {}", event.scoring_team, score.left_team, score.right_team);

        // Add floating notification
        let team_name = match event.scoring_team {
            GoalTeam::Left => "LEFT",
            GoalTeam::Right => "RIGHT",
        };
        notifications
            .notifications
            .push(ScoreNotification::new(format!("⚽ {} GOAL!", team_name), 2.0));

        // Check for match point
        if score.is_match_point() {
            notifications
                .notifications
                .push(ScoreNotification::new("🔥 MATCH POINT!".to_string(), 2.0));
        }

        // Check for match winner
        if let Some(winner) = score.get_winner() {
            match_events.write(MatchFinished { winner: Some(winner) });
            return;
        }

        // Reset player positions after each goal
        reset_events.write(PlayerReset);
    }
}

// Utility function to check if two line segments intersect
pub fn line_segments_intersect(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> bool {
    let d1 = direction(p3, p4, p1);
    let d2 = direction(p3, p4, p2);
    let d3 = direction(p1, p2, p3);
    let d4 = direction(p1, p2, p4);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }

    if d1 == 0.0 && on_segment(p3, p4, p1) {
        return true;
    }
    if d2 == 0.0 && on_segment(p3, p4, p2) {
        return true;
    }
    if d3 == 0.0 && on_segment(p1, p2, p3) {
        return true;
    }
    if d4 == 0.0 && on_segment(p1, p2, p4) {
        return true;
    }

    false
}

pub fn direction(pi: Vec2, pj: Vec2, pk: Vec2) -> f32 {
    (pk.x - pi.x) * (pj.y - pi.y) - (pj.x - pi.x) * (pk.y - pi.y)
}

pub fn on_segment(pi: Vec2, pj: Vec2, pk: Vec2) -> bool {
    pk.x <= pi.x.max(pj.x)
        && pk.x >= pi.x.min(pj.x)
        && pk.y <= pi.y.max(pj.y)
        && pk.y >= pi.y.min(pj.y)
}

// Enhanced reset score system with notification
pub fn reset_score_system(
    mut score: ResMut<Score>,
    mut timer: ResMut<GameTimer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut notifications: ResMut<ScoreNotifications>,
    mut reset_events: EventWriter<PlayerReset>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        if score.left_team > 0 || score.right_team > 0 {
            notifications
                .notifications
                .push(ScoreNotification::new("🔄 MATCH RESET".to_string(), 2.0));
        }
        score.reset();
        timer.remaining_time = timer.match_duration;
        timer.is_finished = false;
        reset_events.write(PlayerReset);
        println!("Match reset!");
    }
}

// New system to handle game timer
pub fn game_timer_system(
    mut timer: ResMut<GameTimer>,
    mut match_events: EventWriter<MatchFinished>,
    mut notifications: ResMut<ScoreNotifications>,
    time: Res<Time>,
    score: Res<Score>,
) {
    if timer.is_finished {
        return;
    }

    timer.remaining_time -= time.delta_secs();
    
    // Time warnings
    let time_left = timer.remaining_time as i32;
    if time_left == 30 && (timer.remaining_time - time_left as f32).abs() < 0.1 {
        notifications.notifications.push(ScoreNotification::new("⏰ 30 SECONDS!".to_string(), 2.0));
    } else if time_left == 10 && (timer.remaining_time - time_left as f32).abs() < 0.1 {
        notifications.notifications.push(ScoreNotification::new("⏰ 10 SECONDS!".to_string(), 2.0));
    }
    
    if timer.remaining_time <= 0.0 {
        timer.remaining_time = 0.0;
        timer.is_finished = true;
        
        // Determine winner by score
        let winner = if score.left_team > score.right_team {
            Some(GoalTeam::Left)
        } else if score.right_team > score.left_team {
            Some(GoalTeam::Right)
        } else {
            None // Draw
        };
        
        match_events.write(MatchFinished { winner });
    }
}

// System to handle match end
pub fn handle_match_finished(
    mut match_events: EventReader<MatchFinished>,
    mut notifications: ResMut<ScoreNotifications>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for event in match_events.read() {
        match &event.winner {
            Some(GoalTeam::Left) => {
                notifications.notifications.push(ScoreNotification::new("🏆 LEFT TEAM WINS!".to_string(), 5.0));
            },
            Some(GoalTeam::Right) => {
                notifications.notifications.push(ScoreNotification::new("🏆 RIGHT TEAM WINS!".to_string(), 5.0));
            },
            None => {
                notifications.notifications.push(ScoreNotification::new("🤝 DRAW!".to_string(), 5.0));
            }
        }
        // Could transition to a game over state here if you had one
    }
}

// Updated GoalPlugin with egui
pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add resources
            .init_resource::<Score>()
            .init_resource::<ScoreNotifications>()
            .init_resource::<GameTimer>()
            // Add events
            .add_event::<GoalScored>()
            .add_event::<MatchFinished>()
            .add_event::<PlayerReset>()
            // Add startup system
            .add_systems(Startup, setup_goal)
            // Add update systems
            .add_systems(
                Update,
                (
                    (handle_goal_scored, reset_score_system, game_timer_system, handle_match_finished).run_if(in_state(AppState::InGame)),
                    (score_ui_system, score_notifications_system)
                        .run_if(in_state(AppState::InGame)),
                ),
            );
    }
}

