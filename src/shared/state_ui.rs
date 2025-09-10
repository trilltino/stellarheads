use crate::shared::state::AppState;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

pub fn launchui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) -> Result {
    if !matches!(current_state.get(), AppState::LaunchMenu) {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Football Heads Game");
            if ui.button("Start Game").clicked() {
                next_state.set(AppState::InGame);
            }
            ui.label(format!("Current State: {:?}", current_state.get()));
        });
    });
    Ok(())
}

pub fn debug_current_gamemode_state(state: Res<State<AppState>>) {
    eprintln!("Current state: {:?}", state.get());
}

pub struct StateUIPlugin;
impl Plugin for StateUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, launchui_system);
        app.add_systems(Update, debug_current_gamemode_state);
    }
}
