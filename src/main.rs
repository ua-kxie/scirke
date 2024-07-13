use bevy::{asset::load_internal_binary_asset, log::LogPlugin, prelude::*, window::PrimaryWindow};
use bevy_egui::EguiPlugin;
use bevyon::BevyonPlugin;
use schematic::SchematicPlugin;

mod bevyon;
mod schematic;
mod input;
pub use bevyon::{FillOptions, StrokeOptions};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (800., 600.).into(),
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                filter: "warn,scirke=trace".into(),
                level: bevy::log::Level::TRACE,
                custom_layer: |_| None,
            }),
    );
    load_internal_binary_asset!(
        app,
        TextStyle::default().font,
        "../assets/MonaspaceNeon-Regular.otf",
        |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
    );
    app.add_plugins(BevyonPlugin)
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
