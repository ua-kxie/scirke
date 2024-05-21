use bevy::prelude::*;

mod background;
mod grid;
mod origin;

pub struct GuidesPlugin;

impl Plugin for GuidesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (background::setup, origin::setup));
        app.add_systems(Update, origin::main);
    }
}
