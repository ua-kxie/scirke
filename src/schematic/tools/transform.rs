use bevy::prelude::*;

use crate::schematic::{
    elements::{LineSegment, LineVertex, Preview, Selected},
    guides::SchematicCursor,
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
        app.add_systems(
            Update,
            main1.run_if(in_state(SchematicToolState::Transform)),
        );
        app.add_systems(
            OnEnter(SchematicToolState::Transform),
            (prep, post_prep).chain(),
        );
        app.add_systems(OnExit(SchematicToolState::Transform), cleanup);
    }
}

/// on entering transform toolstate:
/// delete elements in preview, not selected
/// delete vertices without branches
fn prep(
    mut commands: Commands,
    q: Query<Entity, (Without<Selected>, With<LineSegment>, With<Preview>)>,
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

/// delete vertices without branches
fn post_prep(
    mut commands: Commands,
    mut q: Query<(Entity, &mut LineVertex, &Parent), (Without<Selected>, With<Preview>)>,
    qc: Query<(Entity, &Children), With<SchematicCursor>>,
) {
    let (cursor, cursor_children) = qc.single();
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
    for c in cursor_children {
        let Some(mut entc) = commands.get_entity(*c) else {
            continue;
        };
        entc.remove::<Selected>();
    }
    // find entities to despawn
    // despawn line vertices without valid branches
}

fn cleanup(
    mut commands: Commands,
    st: Res<State<TransformType>>,
    q: Query<Entity, With<Selected>>,
) {
    match st.get() {
        TransformType::Copy => {}
        TransformType::Move => {
            // delete all entities marked as selected
            for e in q.iter() {
                commands.entity(e).despawn();
            }
        }
    }
}

// this tool should be activated more generally through: moving, copying, placing, etc.

fn main1(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut next_toolstate: ResMut<NextState<SchematicToolState>>,
    cursor_children: Query<(Entity, Option<&Children>), With<SchematicCursor>>,
    mut q_transform: Query<(&mut Transform, &GlobalTransform)>,
    mut commands: Commands,
) {
    if buttons.just_pressed(MouseButton::Left) {
        next_toolstate.set(SchematicToolState::Idle);
        // make all children of cursor not such, taking care of transforms, and unmark as preview
        let (cursor_entity, Some(children)) = cursor_children.single() else {
            return;
        };
        commands.entity(cursor_entity).remove_children(children);
        for c in children {
            let (mut t, gt) = q_transform.get_mut(*c).unwrap();
            *t = gt.compute_transform();
            commands.entity(*c).remove::<Preview>();
        }
    }
}

// / main transform system, runs if schematic tool state is transform
// / listen to input events
// fn main(
//     mut e: EventReader<NewSnappedCursor>,
//     c: Query<&SchematicCursor>,
//     mut q: Query<&mut Transform, (With<Selected>, Without<LineSegment>)>,
//     keys: Res<ButtonInput<KeyCode>>,
//     sts: Res<State<SchematicToolState>>,
//     mut stns: ResMut<NextState<SchematicToolState>>,
// ) {
//     let SchematicToolState::Transform(ini_coord) = sts.get() else {
//         return;
//     };
//     let mouse_moved = e.read().last().is_some();
//     let transform_command = keys.any_just_released([KeyCode::KeyX, KeyCode::KeyY]);
//     let valid_coord = c.single().coords.is_some();
//     if (!mouse_moved && !transform_command) || !valid_coord {
//         return;
//     }
//     let new_coord = c.single().coords.clone().unwrap().snapped_world_coords;
//     stns.set(SchematicToolState::Transform(new_coord.as_ivec2()));
//     let scale = Vec3::new(
//         if keys.just_released(KeyCode::KeyX) {
//             -1.0
//         } else {
//             1.0
//         },
//         if keys.just_released(KeyCode::KeyY) {
//             -1.0
//         } else {
//             1.0
//         },
//         1.0,
//     );
//     let rotation = Quat::from_rotation_z(0.0);
//     // let o be origin at 0, 0, c be cursor, and p be point to be transformed
//     // to rotate/flip p about c, dest = transform(p-c) + c
//     // better yet subtract the old cursor position and add the new position to input translation
//     let pos_t = Transform::from_scale(scale).with_rotation(Quat::IDENTITY);
//     for mut t in q.iter_mut() {
//         let dest_pos = pos_t.transform_point(t.translation - ini_coord.as_vec2().extend(0.0))
//             + new_coord.extend(0.0);
//         *t = t
//             .with_rotation(t.rotation + rotation)
//             .with_scale(scale * t.scale)
//             .with_translation(dest_pos);
//     }
// }
