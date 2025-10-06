use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use bevy_egui::{egui, EguiContexts};

#[cfg(not(target_arch = "wasm32"))]
use crate::shared::scoring::{Score, GameTimer, ScoreNotifications};

// ================= STATES =================

#[derive(Clone, Copy, Resource, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    LaunchMenu,
    InGame,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameUI {
    MainMenuUI,
    GameHUD,
    PausedMenuUI,
    ResultUI,
}

// ================= SYSTEMS =================

// WASM-only systems (no egui)
#[cfg(target_arch = "wasm32")]
pub fn launch_screen_system(
    mut next_state: ResMut<NextState<AppState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // Auto-start game after 3 seconds or on any key press
    if keyboard_input.get_just_pressed().next().is_some() {
        next_state.set(AppState::InGame);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn debug_current_gamemode_state(state: Res<State<AppState>>) {
    eprintln!("Current state: {:?}", state.get());
}

// Non-WASM systems (with egui)
#[cfg(not(target_arch = "wasm32"))]
pub fn launch_screen_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Try minimal egui to test context
    let Ok(ctx) = contexts.ctx_mut() else {
        // Fallback - auto start if egui not ready
        next_state.set(AppState::InGame);
        return;
    };

    egui::Window::new("Start Game")
        .fixed_pos([10.0, 10.0])
        .fixed_size([200.0, 100.0])
        .show(ctx, |ui| {
            if ui.button("Start Game").clicked() {
                next_state.set(AppState::InGame);
            }
        });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_fonts(_ctx: EguiContexts) {
    // Font setup disabled for testing
}

#[cfg(not(target_arch = "wasm32"))]
pub fn debug_current_gamemode_state(state: Res<State<AppState>>) {
    eprintln!("Current state: {:?}", state.get());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn score_ui_system(
    _contexts: EguiContexts,
    _score: Res<Score>,
    _timer: Res<GameTimer>,
) {
    // UI stub - not currently implemented
    // Score display is handled via backend/frontend communication
}

#[cfg(not(target_arch = "wasm32"))]
pub fn game_over_ui_system(
    _contexts: EguiContexts,
    _score: Res<Score>,
    _next_state: ResMut<NextState<AppState>>,
) {
    // UI disabled for testing
}

#[cfg(not(target_arch = "wasm32"))]
pub fn notifications_ui_system(
    _contexts: EguiContexts,
    _notifications: ResMut<ScoreNotifications>,
) {
    // UI disabled for testing
}

// ================= PLUGIN =================

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, _app: &mut App) {
        // For WASM builds, we don't add any egui systems
        #[cfg(target_arch = "wasm32")]
        {
            // WASM builds use simple keyboard-based UI
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Desktop builds use full egui UI
            _app.add_systems(OnEnter(AppState::LaunchMenu), setup_fonts)
                .add_systems(Update, launch_screen_system.run_if(in_state(AppState::LaunchMenu)))
                .add_systems(Update, score_ui_system.run_if(in_state(AppState::InGame)))
                .add_systems(Update, game_over_ui_system.run_if(in_state(AppState::GameOver)))
                .add_systems(Update, notifications_ui_system.run_if(in_state(AppState::InGame)));
        }
    }
}