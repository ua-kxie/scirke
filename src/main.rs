use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiPlugin;
use bevyon::BevyonPlugin;
use schematic::SchematicPlugin;

mod bevyon;
mod readable_idgen;
mod schematic;
mod spmanager;
mod console;

pub use bevyon::{FillOptions, StrokeOptions};
use spmanager::SPManagerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SPManagerPlugin)
        .add_plugins(BevyonPlugin)
        .add_plugins(SchematicPlugin)
        .add_systems(Startup, hide_cursor)
        .add_plugins(EguiPlugin)
        .run();
}

fn hide_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let window = &mut primary_window.single_mut();
    // window.cursor.visible = false;
    window.cursor.visible = true;
}
