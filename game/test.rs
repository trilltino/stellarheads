use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .init_state::<AppState>()
        .add_systems(Update, ui_system)
        .run();
}

fn ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    if let Ok(ctx) = contexts.try_ctx_mut() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("🌟 Stellar Heads Test");
                ui.add_space(20.0);

                match current_state.get() {
                    AppState::Menu => {
                        if ui.add(egui::Button::new("🚀 Start Game").min_size(egui::vec2(200.0, 50.0))).clicked() {
                            println!("🎮 BUTTON CLICKED! Starting game...");
                            next_state.set(AppState::Game);
                        }
                    }
                    AppState::Game => {
                        ui.label("🎮 GAME IS RUNNING!");
                        if ui.button("🔙 Back to Menu").clicked() {
                            next_state.set(AppState::Menu);
                        }
                    }
                }

                ui.add_space(10.0);
                ui.label(format!("Current State: {:?}", current_state.get()));
            });
        });
    }
}