use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::shared::scoring::{Score, GameTimer, ScoreNotifications, PlayerInfo};

pub struct EguiInspector;

impl Plugin for EguiInspector {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin)
            .add_systems(Update, enhanced_inspector_ui);
    }
}

fn enhanced_inspector_ui(
    mut contexts: EguiContexts,
    mut score: ResMut<Score>,
    mut timer: ResMut<GameTimer>,
    mut player_info: ResMut<PlayerInfo>,
    notifications: Res<ScoreNotifications>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("🎮 Stellar Heads Inspector")
        .default_width(400.0)
        .show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                
                // Game State Section
                ui.heading("🎯 Game State");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Left Score:");
                    ui.add(egui::DragValue::new(&mut score.left_team).range(0..=10));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Right Score:");
                    ui.add(egui::DragValue::new(&mut score.right_team).range(0..=10));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Time Remaining:");
                    ui.add(egui::DragValue::new(&mut timer.remaining_time).range(0.0..=300.0).suffix("s"));
                });
                
                if ui.button("🔄 Reset Score").clicked() {
                    score.reset();
                    timer.remaining_time = timer.match_duration;
                    timer.is_finished = false;
                }
                
                ui.separator();
                
                // Player Info Section
                ui.heading("👤 Player Info");
                ui.text_edit_singleline(&mut player_info.username);
                ui.text_edit_singleline(&mut player_info.wallet_address);
                
                ui.separator();
                
                // Notifications Section
                ui.heading("📢 Active Notifications");
                for notification in &notifications.notifications {
                    ui.label(format!("⏱️ {:.1}s: {}", notification.timer, notification.text));
                }
                
                ui.separator();
                
                // World Inspector Section
                ui.heading("🌍 World Inspector");
                egui::CollapsingHeader::new("Entities").show(ui, |ui| {
                    ui.label("Entity inspector requires mutable world access");
                });

                egui::CollapsingHeader::new("Resources").show(ui, |ui| {
                    ui.label("Resource inspector requires mutable world access");
                });
            });
        });
}

