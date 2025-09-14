use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use crate::shared::scoring::{Score, GameTimer, ScoreNotifications};

// ================= STATES =================

#[derive(Clone, Copy, Resource, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    LaunchMenu,
    InGame,
    Paused,
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
            AppState::Paused => Some(GameUI::PausedMenuUI),
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
    let handle: Handle<Image> = asset_server.load("art/Splash.png");
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
            let nominal_size = ui.available_size();
            // Use screen rect size for full screen splash
            let image = egui::Image::new((id, screen_rect.size()))
                .fit_to_exact_size(screen_rect.size())
                .rounding(egui::Rounding::ZERO);

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



pub struct StateUIPlugin;
impl Plugin for StateUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LaunchMenu), on_enter_load_splash);
        app.add_systems(EguiPrimaryContextPass, launch_menu_system.run_if(in_state(AppState::LaunchMenu)));
        app.add_systems(Update, debug_current_gamemode_state);
    }
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,setup_fonts);
        app.add_systems(
            EguiPrimaryContextPass,
            (score_ui_system, score_notifications_system)
                .run_if(in_state(AppState::InGame)),
        );
    }
}
