use bevy::prelude::*;

pub mod console;
mod params_editor;

pub struct SchematicUiPlugin;

impl Plugin for SchematicUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(console::ConsolePlugin);
        app.add_systems(Update, params_editor::params_ui);
    }
}
