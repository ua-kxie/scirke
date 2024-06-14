use std::f32::consts::PI;

use bevy::prelude::*;

use crate::schematic::{
    elements::{LineVertex, Preview, Selected},
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
        // app.add_systems(Startup, setup);
        app.add_systems(Update, main.run_if(in_state(SchematicToolState::Transform)));
        app.add_systems(
            OnEnter(SchematicToolState::Transform),
            (prep, post_prep).chain(),
        );
        // app.add_systems(OnExit(SchematicToolState::Transform), cleanup);
    }
}

/// on entering transform toolstate:
/// delete elements in preview, not selected
/// delete vertices without branches
fn prep(
    mut commands: Commands,
    q: Query<Entity, (Without<Selected>, With<Preview>)>,
    qc: Query<Entity, With<SchematicCursor>>,
) {
    let cursor = qc.single();
    // despawn linesegments not selected
    commands
        .entity(cursor)
        .remove_children(&q.iter().collect::<Box<[Entity]>>());
    for e in q.iter() {
        commands.entity(e).despawn();
    }
    // despawn line vertices without valid branches
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
    q_selected: Query<Entity, With<Selected>>,
    mut notify_changed: EventWriter<SchematicChanged>,
) {
    let (cursor_entity, Some(children)) = cursor_children.single() else {
        return;
    };
    if buttons.just_pressed(MouseButton::Left) {
        next_toolstate.set(SchematicToolState::Idle);
        // make all children of cursor not such, taking care of transforms, and unmark as preview
        commands.entity(cursor_entity).remove_children(children);
        for c in children {
            let (mut t, gt) = q_transform.get_mut(*c).unwrap();
            *t = gt.compute_transform();
            commands.entity(*c).remove::<Preview>();
        }
        match st.get() {
            TransformType::Copy => {}
            TransformType::Move => {
                // delete all entities marked as selected
                for e in q_selected.iter() {
                    commands.entity(e).despawn();
                }
            }
        }
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
