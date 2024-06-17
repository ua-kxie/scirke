use std::f32::consts::PI;

use bevy::prelude::*;

use crate::schematic::{
    elements::{LineVertex, PickableElement, Preview, Selected, SpDeviceId},
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
        app.add_systems(Update, main.run_if(in_state(SchematicToolState::Transform)));
        app.add_systems(OnEnter(SchematicToolState::Transform), prep.chain());
        app.add_systems(OnExit(SchematicToolState::Transform), clear_cursor_children);
    }
}

/// this system clears cursor children
/// runs upon entering idle state (exit transform tool)
fn clear_cursor_children(
    mut commands: Commands,
    q: Query<(Entity, &Children), With<SchematicCursor>>,
) {
    let Ok((parent, cursor_children)) = q.get_single() else {
        return;
    };
    commands.entity(parent).remove_children(&cursor_children);
    for e in cursor_children {
        commands.entity(*e).despawn();
    }
}

/// on entering transform toolstate:
/// delete pickable not selected elements in preview,
/// remove from cursor children: unpickable elements in preview
fn prep(
    mut commands: Commands,
    qc: Query<Entity, With<SchematicCursor>>,
    q_unpicked: Query<Entity, (Without<Selected>, With<Preview>, With<PickableElement>)>,
    q_unpickable: Query<Entity, (With<Preview>, Without<PickableElement>)>,
) {
    let cursor = qc.single();
    // despawn pickable entities in preview not tagged as selected
    commands
        .entity(cursor)
        .remove_children(&q_unpicked.iter().collect::<Box<[Entity]>>());
    for e in q_unpicked.iter() {
        commands.entity(e).despawn();
    }

    // remove non-pickable (ports) from cursor
    commands
        .entity(cursor)
        .remove_children(&q_unpickable.iter().collect::<Box<[Entity]>>());
}

/// all SchematicElements are copied in
/// [`prep`] may leave some straggling vertices
/// finally, remove Selected tag
fn post_prep(
    mut commands: Commands,
    mut q: Query<(Entity, &mut LineVertex, &Parent), (Without<Selected>, With<Preview>)>,
    qc: Query<(Entity, Option<&Children>), With<SchematicCursor>>,
) {
    let (cursor, opt_cursor_children) = qc.single();
    for (e, mut lv, parent) in q.iter_mut() {
        if parent.get() == cursor {
            (*lv).branches = lv
                .branches
                .iter()
                .filter_map(|ls| {
                    if commands.get_entity(*ls).is_some() {
                        Some(*ls)
                    } else {
                        None
                    }
                })
                .collect();

            if lv.branches.is_empty() {
                commands.entity(cursor).remove_children(&[e]);
                commands.entity(e).despawn();
            }
        }
    }
    opt_cursor_children.map(|x| {
        x.iter().map(|e| {
            commands.entity(*e).remove::<Selected>();
        })
    });
    // find entities to despawn
    // despawn line vertices without valid branches
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
    q_previews: Query<Entity, With<Preview>>,
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
                    commands.entity(*c).remove::<SpDeviceId>();
                }
            }
            TransformType::Move => {
                // delete all entities not in preview marked as selected
                for e in q_selected_not_preview.iter() {
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
        q_previews.iter().for_each(|e| {
            commands.entity(e).remove::<Preview>();
        });

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
