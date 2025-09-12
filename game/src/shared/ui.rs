use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::shared::state::AppState;
use crate::shared::scoring::{Score, GameTimer, ScoreNotifications};

// ================= UI Systems =================

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
        return;
    };
    let screen_rect = ctx.screen_rect();

    // Update timers and remove expired notifications
    notifications.notifications.retain_mut(|notif| {
        notif.timer -= time.delta_secs();
        notif.timer > 0.0
    });

    // Display notifications
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

// ================= UI Plugin =================

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (score_ui_system, score_notifications_system)
                .run_if(in_state(AppState::InGame)),
        );
    }
}