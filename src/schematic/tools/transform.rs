use bevy::prelude::*;

use crate::schematic::{
    elements::{LineSegment, Selected},
    guides::{NewSnappedCursor, SchematicCursor},
};

use super::SchematicToolState;

pub struct TransformToolPlugin;

impl Plugin for TransformToolPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.add_systems(Update, main);
    }
}

// this tool should be activated more generally through: moving, copying, placing, etc.

/// main transform system, runs if schematic tool state is transform
/// listen to input events
fn main(
    mut e: EventReader<NewSnappedCursor>,
    c: Query<&SchematicCursor>,
    mut q: Query<&mut Transform, (With<Selected>, Without<LineSegment>)>,
    keys: Res<ButtonInput<KeyCode>>,
    sts: Res<State<SchematicToolState>>,
    mut stns: ResMut<NextState<SchematicToolState>>,
) {
    let SchematicToolState::Transform(ini_coord) = sts.get() else {
        return;
    };
    let mouse_moved = e.read().last().is_some();
    let transform_command = keys.any_just_released([KeyCode::KeyX, KeyCode::KeyY]);
    let valid_coord = c.single().coords.is_some();
    if (!mouse_moved && !transform_command) || !valid_coord {
        return;
    }
    let new_coord = c.single().coords.clone().unwrap().snapped_world_coords;
    stns.set(SchematicToolState::Transform(new_coord.as_ivec2()));
    let scale = Vec3::new(
        if keys.just_released(KeyCode::KeyX) {
            -1.0
        } else {
            1.0
        },
        if keys.just_released(KeyCode::KeyY) {
            -1.0
        } else {
            1.0
        },
        1.0,
    );
    let rotation = Quat::from_rotation_z(0.0);
    // let o be origin at 0, 0, c be cursor, and p be point to be transformed
    // to rotate/flip p about c, dest = transform(p-c) + c
    // better yet subtract the old cursor position and add the new position to input translation
    let pos_t = Transform::from_scale(scale).with_rotation(Quat::IDENTITY);
    for mut t in q.iter_mut() {
        let dest_pos = pos_t.transform_point(t.translation - ini_coord.as_vec2().extend(0.0))
            + new_coord.extend(0.0);
        *t = t
            .with_rotation(t.rotation + rotation)
            .with_scale(scale * t.scale)
            .with_translation(dest_pos);
    }
}
