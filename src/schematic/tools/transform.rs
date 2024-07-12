use std::f32::consts::PI;

use bevy::prelude::*;

use crate::schematic::{
    electrical::{self, NetId, Preview, SchematicElement, Selected, SpDeviceId},
    guides::SchematicCursor,
    SchematicChanged,
};

use super::SchematicToolState;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransformType {
    #[default]
    Copy, // do nothing after persisting preview
    Move, // delete source after persisting preview
}

pub struct TransformToolPlugin;

impl Plugin for TransformToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TransformType>();
        app.add_systems(
            PreUpdate,
            main.run_if(in_state(SchematicToolState::Transform)),
        );
        // app.add_systems(OnEnter(SchematicToolState::Transform), prep);
        app.add_systems(OnExit(SchematicToolState::Transform), clear_cursor_children);
    }
}

/// this system clears cursor children
/// runs upon exiting transform tool
fn clear_cursor_children(
    mut commands: Commands,
    q: Query<(Entity, &Children), With<SchematicCursor>>,
) {
    let Ok((parent, cursor_children)) = q.get_single() else {
        return;
    };
    commands.entity(parent).remove_children(&cursor_children);
    for e in cursor_children {
        debug!("deleting cursor children");
        commands.entity(*e).despawn();
    }
}

// this tool should be activated more generally through: moving, copying, placing, etc.

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut next_toolstate: ResMut<NextState<SchematicToolState>>,
    cursor_children: Query<(Entity, Option<&Children>), With<SchematicCursor>>,
    mut q_transform: Query<(&mut Transform, &GlobalTransform)>,
    mut commands: Commands,
    st: Res<State<TransformType>>,
    q_selected_not_preview: Query<Entity, (With<Selected>, Without<Preview>)>,
    q_previews: Query<Entity, (With<SchematicElement>, With<Preview>)>,
    mut notify_changed: EventWriter<SchematicChanged>,
) {
    let (cursor_entity, Some(children)) = cursor_children.single() else {
        return;
    };
    if buttons.just_pressed(MouseButton::Left) {
        match st.get() {
            TransformType::Copy => {
                // delete spid component on new copies
                for c in children.iter() {
                    commands.entity(*c).remove::<SpDeviceId>().remove::<NetId>();
                }
            }
            TransformType::Move => {
                // delete all entities not in preview marked as selected, e.g. the entities originally selected for move
                for e in q_selected_not_preview.iter() {
                    debug!("deleting selected, not preview entities (move tool, delete source entities)");
                    commands.entity(e).despawn();
                }
            }
        }

        next_toolstate.set(SchematicToolState::Idle);
        // make all children of cursor not such, taking care of transforms
        commands.entity(cursor_entity).remove_children(children);
        for c in children {
            let (mut t, gt) = q_transform.get_mut(*c).unwrap();
            *t = gt.compute_transform();
        }
        // unmark all entites as preview
        electrical::persist_preview(&mut commands, &q_previews);

        notify_changed.send(SchematicChanged);
        return; // ignore other commands because its effects were never shown to user
    }
    let mut transform = Transform::IDENTITY;
    if keys.just_pressed(KeyCode::KeyR) {
        transform.rotate_z(if keys.pressed(KeyCode::ShiftLeft) {
            -PI / 2.0
        } else {
            PI / 2.0
        });
    }
    if keys.just_pressed(KeyCode::KeyX) {
        transform.scale = transform.scale * Vec3::new(-1.0, 1.0, 1.0);
    }
    if keys.just_pressed(KeyCode::KeyY) {
        transform.scale = transform.scale * Vec3::new(1.0, -1.0, 1.0);
    }
    if transform != Transform::IDENTITY {
        let Ok((mut t, _)) = q_transform.get_mut(cursor_entity) else {
            return;
        };
        t.scale = transform.scale * t.scale;
        t.rotation = transform.rotation * t.rotation;
    }
}
