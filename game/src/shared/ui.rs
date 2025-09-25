use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use crate::shared::scoring::{Score, GameTimer, ScoreNotifications, GoalTeam};

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

impl ComputedStates for GameUI {
    type SourceStates = AppState;

    fn compute(source_states: Self::SourceStates) -> Option<Self> {
        match source_states {
            AppState::LaunchMenu => Some(GameUI::MainMenuUI),
            AppState::InGame => Some(GameUI::GameHUD),
            AppState::GameOver => Some(GameUI::ResultUI),
        }
    }
}

#[derive(Resource)]
pub struct SplashImage {
    handle: Handle<Image>,
    egui_id: Option<egui::TextureId>,
}

pub fn on_enter_load_splash(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<Image> = asset_server.load("art/splashscreen.png");
    commands.insert_resource(SplashImage { handle, egui_id: None });
}

pub fn launch_menu_system(
    mut contexts: EguiContexts,
    splash: Option<ResMut<SplashImage>>,
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
    asset_server: Res<AssetServer>,
) {
    if *current_state.get() != AppState::LaunchMenu {
        return;
    }
    let mut splash = match splash {
        Some(s) => s,
        None => return, // no splash loaded yet
    };

    // Check if the image asset is actually loaded before trying to use it
    if !matches!(asset_server.load_state(&splash.handle), bevy::asset::LoadState::Loaded) {
        return; // Wait for image to load
    }

    if splash.egui_id.is_none() {
        let id = contexts.add_image(splash.handle.clone());
        splash.egui_id = Some(id);
    }

    if let Ok(ctx) = contexts.ctx_mut() {
        let id = splash.egui_id.expect("egui texture id present");
        egui::CentralPanel::default().show(ctx, |ui| {
            let screen_rect = ui.ctx().screen_rect();
            let _nominal_size = ui.available_size();
            // Use screen rect size for full screen splash
            let image = egui::Image::new((id, screen_rect.size()))
                .fit_to_exact_size(screen_rect.size())
                .corner_radius(egui::CornerRadius::ZERO);

            // paint full screen with no filtering to prevent compression artifacts
            image.paint_at(ui, screen_rect);

            // Position Play Game button in lower center like splash screen
            ui.vertical_centered(|ui| {
                // Push button to lower portion of screen
                ui.add_space(400.0);

                // Style button to match splash screen - green rounded button
                let button = egui::Button::new(
                    egui::RichText::new("PLAY GAME")
                        .size(24.0)
                        .color(egui::Color32::WHITE)
                )
                .fill(egui::Color32::from_rgb(34, 139, 34)) // Forest green
                .min_size(egui::vec2(250.0, 60.0)); // Match splash button size

                if ui.add(button).clicked() {
                    next_state.set(AppState::InGame);
                }
            });
        });
    }
}




pub fn setup_fonts(mut ctx: EguiContexts) {
    use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};
    use std::sync::Arc;

    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "PressStart2P".to_string(),
      Arc::new(FontData::from_static(include_bytes!("../../assets/PressStart2P-Regular.ttf"))),
    );

    fonts.families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "PressStart2P".to_string());

    fonts.families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "PressStart2P".to_string());

    ctx.ctx_mut()
        .expect("Failed to get egui context")
        .set_fonts(fonts);
}


pub fn debug_current_gamemode_state(state: Res<State<AppState>>) {
    eprintln!("Current state: {:?}", state.get());
}


pub fn score_ui_system(
    mut contexts: EguiContexts,
    score: Res<Score>,
    timer: Res<GameTimer>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Score")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(egui::pos2(533.0, 20.0))
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
 
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", score.left_team))
                                .size(48.0)
                                .color(egui::Color32::from_rgb(120, 170, 255))
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
                                .color(egui::Color32::from_rgb(255, 120, 120))
                                .strong(),
                        );
                    });
                });
 
                ui.add_space(5.0);
 
                let remaining = timer.remaining_time.max(0.0).floor() as i32;
                let minutes = remaining / 60;
                let seconds = remaining % 60;
                let timer_color = if timer.remaining_time < 30.0 {
                    egui::Color32::from_rgb(255, 120, 120)
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
 
    egui::Window::new("LeftTeam")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(egui::pos2(50.0, 60.0))
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("LEFT TEAM")
                    .size(14.0)
                    .color(egui::Color32::from_rgb(120, 170, 255))
                    .strong(),
            );
        });
 
    egui::Window::new("RightTeam")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(egui::pos2(1246.0, 60.0))
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("RIGHT TEAM")
                    .size(14.0)
                    .color(egui::Color32::from_rgb(255, 120, 120))
                    .strong(),
            );
        });
}
 
pub fn score_notifications_system(
    mut contexts: EguiContexts,
    mut notifications: ResMut<ScoreNotifications>,
    time: Res<Time>,
) {

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return, // nothing to draw this frame
    };

    for notif in notifications.notifications.iter_mut() {
        notif.timer -= time.delta_secs();
    }

    notifications.notifications.retain(|n| n.timer > 0.0);

    for (i, notif) in notifications.notifications.iter().enumerate() {
        let ratio = (notif.timer / notif.max_time).clamp(0.0, 1.0);
        let alpha = (ratio * 255.0).round().clamp(0.0, 255.0) as u8;
        let y_offset = 150.0 + (i as f32 * 50.0);

        egui::Window::new(format!("notification_{}", i))
            .title_bar(false)
            .resizable(false)
            .fixed_pos(egui::pos2(583.0, y_offset))
            .frame(egui::Frame::NONE)
            .show(&*ctx, |ui| {
                ui.label(
                    egui::RichText::new(&notif.text)
                        .size(36.0)
                        .color(egui::Color32::from_rgba_unmultiplied(255, 215, 0, alpha))
                        .strong(),
                );
            });
    }
}

// ================= GAME OVER UI =================

#[derive(Resource)]
pub struct MatchResult {
    pub winner: Option<GoalTeam>,
    pub final_score: (u32, u32),
}

pub fn store_match_result(
    mut commands: Commands,
    score: Res<Score>,
    timer: Res<GameTimer>,
) {
    // Determine winner
    let winner = if score.left_team >= 5 {
        Some(GoalTeam::Left)
    } else if score.right_team >= 5 {
        Some(GoalTeam::Right)
    } else if timer.is_finished {
        // Time's up, determine by score
        if score.left_team > score.right_team {
            Some(GoalTeam::Left)
        } else if score.right_team > score.left_team {
            Some(GoalTeam::Right)
        } else {
            None // Draw
        }
    } else {
        None
    };

    commands.insert_resource(MatchResult {
        winner,
        final_score: (score.left_team, score.right_team),
    });
}

pub fn game_over_ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut score: ResMut<Score>,
    mut timer: ResMut<GameTimer>,
    match_result: Option<Res<MatchResult>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let result = match match_result {
        Some(result) => result,
        None => return,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        let screen_size = ui.available_size();

        ui.vertical_centered(|ui| {
            ui.add_space(screen_size.y * 0.25);

            // Game Over Title
            ui.label(
                egui::RichText::new("🎮 GAME OVER")
                    .size(48.0)
                    .color(egui::Color32::GOLD)
                    .strong(),
            );

            ui.add_space(30.0);

            // Winner announcement
            let (winner_text, winner_color) = match &result.winner {
                Some(GoalTeam::Left) => ("🏆 YOU WIN!", egui::Color32::from_rgb(120, 255, 120)),
                Some(GoalTeam::Right) => ("😞 YOU LOSE!", egui::Color32::from_rgb(255, 120, 120)),
                None => ("🤝 DRAW!", egui::Color32::WHITE),
            };

            ui.label(
                egui::RichText::new(winner_text)
                    .size(36.0)
                    .color(winner_color)
                    .strong(),
            );

            ui.add_space(20.0);

            // Final Score
            ui.label(
                egui::RichText::new(format!("Final Score: {} - {}", result.final_score.0, result.final_score.1))
                    .size(24.0)
                    .color(egui::Color32::WHITE),
            );

            ui.add_space(40.0);

            // Play Again Button
            let play_again_button = egui::Button::new(
                egui::RichText::new("🔄 PLAY AGAIN")
                    .size(24.0)
                    .color(egui::Color32::WHITE)
            )
            .fill(egui::Color32::from_rgb(34, 139, 34)) // Forest green
            .min_size(egui::vec2(250.0, 60.0));

            if ui.add(play_again_button).clicked() {
                // Reset game state
                score.reset();
                timer.remaining_time = timer.match_duration;
                timer.is_finished = false;

                // Return to game
                next_state.set(AppState::InGame);
                println!("🔄 Starting new game...");
            }

            ui.add_space(20.0);

            // Main Menu Button
            let main_menu_button = egui::Button::new(
                egui::RichText::new("🏠 MAIN MENU")
                    .size(20.0)
                    .color(egui::Color32::WHITE)
            )
            .fill(egui::Color32::from_rgb(139, 69, 19)) // Brown
            .min_size(egui::vec2(200.0, 50.0));

            if ui.add(main_menu_button).clicked() {
                // Reset game state
                score.reset();
                timer.remaining_time = timer.match_duration;
                timer.is_finished = false;

                // Return to main menu
                next_state.set(AppState::LaunchMenu);
                println!("🏠 Returning to main menu...");
            }
        });
    });
}

pub struct StateUIPlugin;
impl Plugin for StateUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameOver), store_match_result)
           .add_systems(Update, debug_current_gamemode_state);

        // Only add EGUI systems for native builds
        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(OnEnter(AppState::LaunchMenu), on_enter_load_splash)
               .add_systems(EguiPrimaryContextPass, launch_menu_system.run_if(in_state(AppState::LaunchMenu)))
               .add_systems(EguiPrimaryContextPass, game_over_ui_system.run_if(in_state(AppState::GameOver)));
        }
    }
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // Only add EGUI systems for native builds
        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(Update, setup_fonts);
            app.add_systems(
                Update,
                (score_ui_system, score_notifications_system)
                    .run_if(in_state(AppState::InGame)),
            );
        }
        // WASM version - no EGUI systems needed
        #[cfg(target_arch = "wasm32")]
        {
            // Add any WASM-specific UI systems here if needed in the future
        }
    }
}
