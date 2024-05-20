use bevy::prelude::*;

use super::{camera::SchematicCamera, cursor::SchematicCursor, SnapSet};

#[derive(Component)]
struct InfoTextMarker;

pub struct InfoPlugin;

impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(PostUpdate, update.after(SnapSet));
    }
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
        InfoTextMarker,
    ));
}

fn update(
    mut text: Query<&mut Text, With<InfoTextMarker>>,
    cursor: Query<&SchematicCursor>,
    projection: Query<&OrthographicProjection, With<SchematicCamera>>,
) {
    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;
    *text = "".to_string();
    
    text.push_str(&format!("scale: {:.2e}; ", projection.single().scale));
    if let Some(coords) = &cursor.single().coords {
        text.push_str(&format!("x: {:+03}; y: {:+03}; ", coords.snapped_world_coords.x, coords.snapped_world_coords.y))
    }

}
