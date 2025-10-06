use bevy::prelude::*;
use crate::shared::{AppState, config::ScoringConfig};
use shared::dto::game::{GameResult, MatchResult};

// ================= HTTP Client for Direct Communication =================

// ================= Player Information =================

#[derive(Resource, Default)]
pub struct PlayerInfo {
    pub username: String,
    pub wallet_address: String,
}

// ================= Game Session Tracking =================

gtgg#[derive(Resource)]
pub struct GameSession {
    pub session_id: String,
}

impl Default for GameSession {
    fn default() -> Self {
        Self::new()
    }
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}


// ================= Score Management =================

#[derive(Resource, Default)]
pub struct Score {
    pub left_team: u32,
    pub right_team: u32,
    pub high_score: u32,
}

impl Score {
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

    pub fn get_winner(&self, winning_score: i32) -> Option<GoalTeam> {
        if self.left_team >= winning_score as u32 {
            Some(GoalTeam::Left)
        } else if self.right_team >= winning_score as u32 {
            Some(GoalTeam::Right)
        } else {
            None
        }
    }

    pub fn is_match_point(&self, winning_score: i32) -> bool {
        let match_point = (winning_score - 1) as u32;
        self.left_team >= match_point || self.right_team >= match_point
    }
}

// ================= Game Timer =================

#[derive(Resource)]
pub struct GameTimer {
    pub remaining_time: f32,
    pub match_duration: f32,
    pub is_finished: bool,
}

impl GameTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            remaining_time: duration,
            match_duration: duration,
            is_finished: false,
        }
    }
}

impl Default for GameTimer {
    fn default() -> Self {
        Self::new(120.0) // Fallback to 2 minutes
    }
}

// ================= Score Notifications =================

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

// ================= Events =================

#[derive(Event)]
pub struct GoalScored {
    pub goal_position: Vec3,
    pub scoring_team: GoalTeam,
}

#[derive(Event)]
pub struct MatchFinished {
    pub winner: Option<GoalTeam>,
}

#[derive(Event)]
pub struct PlayerReset;

// ================= Goal Team =================

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum GoalTeam {
    Left,
    Right,
}

// ================= Scoring Systems =================

pub fn handle_goal_scored(
    scoring_config: Res<ScoringConfig>,
    mut score: ResMut<Score>,
    mut goal_events: EventReader<GoalScored>,
    mut notifications: ResMut<ScoreNotifications>,
    mut reset_events: EventWriter<PlayerReset>,
    mut match_events: EventWriter<MatchFinished>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let event_count = goal_events.len();
        if event_count > 0 {
            println!("🎯 SCORING SYSTEM: Received {event_count} GoalScored events!");
        }
    }

    for event in goal_events.read() {
        #[cfg(not(target_arch = "wasm32"))]
        println!("🎯 PROCESSING GoalScored event for {:?} team at {:?}", event.scoring_team, event.goal_position);

        score.add_point(event.scoring_team.clone());

        #[cfg(not(target_arch = "wasm32"))]
        println!("🎯 ⚽ GOAL SCORED by {:?}! NEW SCORE: {} - {}", event.scoring_team, score.left_team, score.right_team);

        // Add floating notification
        let team_name = match event.scoring_team {
            GoalTeam::Left => "LEFT",
            GoalTeam::Right => "RIGHT",
        };
        notifications
            .notifications
            .push(ScoreNotification::new(format!("⚽ {team_name} GOAL!"), 2.0));

        // Check for match point
        if score.is_match_point(scoring_config.winning_score) {
            notifications
                .notifications
                .push(ScoreNotification::new("🔥 MATCH POINT!".to_string(), 2.0));
        }

        // Check for match winner
        if let Some(winner) = score.get_winner(scoring_config.winning_score) {
            match_events.write(MatchFinished { winner: Some(winner) });
            return;
        }

        // Reset player positions after each goal
        reset_events.write(PlayerReset);
    }
}

pub fn reset_score_system(
    mut score: ResMut<Score>,
    mut timer: ResMut<GameTimer>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut notifications: ResMut<ScoreNotifications>,
    mut reset_events: EventWriter<PlayerReset>,
) {
    // Only check keyboard input if the resource is available (not in headless tests)
    if let Some(keyboard) = keyboard {
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

            #[cfg(not(target_arch = "wasm32"))]
            println!("Match reset!");
        }
    }
}

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

pub fn handle_match_finished(
    mut match_events: EventReader<MatchFinished>,
    mut notifications: ResMut<ScoreNotifications>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for event in match_events.read() {
        match &event.winner {
            Some(GoalTeam::Left) => {
                notifications.notifications.push(ScoreNotification::new("🏆 LEFT TEAM WINS!".to_string(), 3.0));
            },
            Some(GoalTeam::Right) => {
                notifications.notifications.push(ScoreNotification::new("🏆 RIGHT TEAM WINS!".to_string(), 3.0));
            },
            None => {
                notifications.notifications.push(ScoreNotification::new("🤝 DRAW!".to_string(), 3.0));
            }
        }

        // Transition to GameOver state after a short delay for notifications
        next_state.set(AppState::GameOver);

        #[cfg(not(target_arch = "wasm32"))]
        println!("🏁 Match finished, transitioning to GameOver state");
    }
}

pub fn send_game_result_system(
    mut match_events: EventReader<MatchFinished>,
    score: Res<Score>,
    timer: Res<GameTimer>,
    player_info: Res<PlayerInfo>,
    game_session: Option<Res<GameSession>>,
) {
    for event in match_events.read() {
        if player_info.username.is_empty() || player_info.wallet_address.is_empty() {
            println!("⚠️ Player info not set, skipping game result submission");
            continue;
        }

        let session_id = match &game_session {
            Some(session) => session.session_id.clone(),
            None => {
                println!("⚠️ No game session found, creating temporary ID");
                uuid::Uuid::new_v4().to_string()
            }
        };

        // Determine match result from LOCAL PLAYER's perspective
        // Local player is left team (blue), AI opponent is right team (red)
        let player_result = match &event.winner {
            Some(GoalTeam::Left) => MatchResult::Win,   // Local player wins
            Some(GoalTeam::Right) => MatchResult::Loss, // Local player loses to AI
            None => MatchResult::Draw,
        };

        // Create game result with individual instance data
        let game_result = GameResult::new(
            player_info.username.clone(),        // Local player username
            player_info.wallet_address.clone(),  // Local player wallet
            player_result,                       // Win/Loss/Draw from player's perspective
            score.left_team,                     // Local player's score (left team)
            score.right_team,                    // AI opponent's score (right team)
            timer.match_duration - timer.remaining_time,
            session_id,                          // Unique game session ID
        ).with_game_mode("single_player_vs_ai".to_string());

        // Send game result directly to backend via HTTP
        println!("🎮 Sending game result to backend: {game_result:?}");

        // Spawn task to submit game result
        let game_result_clone = game_result.clone();

        // Send game result via PostMessage for iframe communication
        #[cfg(target_arch = "wasm32")]
        {
            if let Err(e) = send_game_result_via_postmessage(&game_result_clone) {
                web_sys::console::log_1(&format!("Failed to send game result via PostMessage: {:?}", e).into());
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // For native builds, just log (could add file logging here)
            println!("Game result: {game_result_clone:?}");
        }
    }
}

// ================= Setup System =================

pub fn setup_player_info(mut player_info: ResMut<PlayerInfo>) {
    if player_info.username.is_empty() {
        // Set default player info for testing
        player_info.username = "test_player".to_string();
        player_info.wallet_address = "GCKFBEIYTKP33TO3QLCCKMXOMVK7X4PYC7_TEST_ADDRESS".to_string();
        println!("🎮 Default player info set: {}", player_info.username);
    }
}

/// System that runs when entering InGame state - creates a new game session
pub fn create_game_session(
    mut commands: Commands,
    scoring_config: Res<ScoringConfig>,
) {
    let session = GameSession::new();

    #[cfg(not(target_arch = "wasm32"))]
    println!("🎲 New game session created: {}", session.session_id);

    commands.insert_resource(session);

    // Initialize timer with configured duration
    let timer = GameTimer::new(scoring_config.match_duration_seconds);
    commands.insert_resource(timer);
}

// ================= Scoring Plugin =================

pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add resources
            .init_resource::<Score>()
            .init_resource::<ScoreNotifications>()
            .init_resource::<GameTimer>()
            .init_resource::<PlayerInfo>()
            // Add events
            .add_event::<GoalScored>()
            .add_event::<MatchFinished>()
            .add_event::<PlayerReset>()
            // Add systems
            .add_systems(Startup, setup_player_info)
            .add_systems(OnEnter(AppState::InGame), create_game_session)
            .add_systems(
                Update,
                (
                    handle_goal_scored,
                    reset_score_system,
                    game_timer_system,
                    handle_match_finished,
                    send_game_result_system, // Direct HTTP communication
                ).run_if(in_state(AppState::InGame)),
            );
    }
}

// ================= PostMessage Communication =================

#[cfg(target_arch = "wasm32")]
fn send_game_result_via_postmessage(game_result: &GameResult) -> Result<(), Box<dyn std::error::Error>> {
    use wasm_bindgen::JsValue;
    use serde_json;
    use web_sys;

    // Create structured message for parent window
    let message = serde_json::json!({
        "type": "game_result",
        "timestamp": chrono::Utc::now().timestamp(),
        "data": {
            "player_address": game_result.player_wallet_address,
            "player_username": game_result.player_username,
            "won": matches!(game_result.player_result, shared::dto::game::MatchResult::Win),
            "score_left": game_result.player_score,
            "score_right": game_result.opponent_score,
            "match_duration_seconds": game_result.duration_seconds
        }
    });

    // Get window object and parent reference
    let window = web_sys::window()
        .ok_or("No window object available")?;

    let parent = window.parent()
        .map_err(|_| "Failed to get parent window")?
        .ok_or("No parent window available")?;

    // Convert message to JsValue
    let js_message = JsValue::from_str(&message.to_string());

    // Send message to parent window with wildcard origin (can be restricted for security)
    parent.post_message(&js_message, "*")
        .map_err(|e| format!("PostMessage failed: {:?}", e))?;

    web_sys::console::log_1(&"✅ Game result sent to parent via PostMessage".into());

    Ok(())
}