use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct EguiInspector;

impl Plugin for EguiInspector {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldInspectorPlugin::new());
    }
}

