use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_save::prelude::*;
use transform::TransformType;

mod sel;
mod transform;
mod wire;

use super::{
    elements::{
        Device, DeviceType, ElementsRes, LineSegment, LineVertex, NewDevice, Preview,
        SchematicElement, Selected,
    },
    guides::SchematicCursor,
    material::SchematicMaterial,
};
pub use sel::{NewPickingCollider, PickingCollider, SelectEvt};

const WIRE_TOOL_KEY: KeyCode = KeyCode::KeyW;
const MOVE_KEY: KeyCode = KeyCode::KeyM;
const COPY_KEY: KeyCode = KeyCode::KeyC;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum SchematicToolState {
    #[default]
    Idle, // also select
    Wiring, // for drawing wires
    Transform, // moving elements around,
            // Label,   // wire/net labeling
            // Comment, // plain text comment with basic formatting options
}

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            wire::WireToolPlugin,
            sel::SelToolPlugin,
            transform::TransformToolPlugin,
        ));
        app.add_systems(Update, exclusive_main);
        app.add_systems(OnEnter(SchematicToolState::Idle), clear_cursor_children);
        app.init_state::<SchematicToolState>();
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

fn exclusive_main(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let curr_toolst = world.resource::<State<SchematicToolState>>();

    if keys.just_released(KeyCode::Escape) {
        let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
        next_toolst.set(SchematicToolState::Idle);
        return;
    }
    match curr_toolst.get() {
        SchematicToolState::Idle => {
            let is_move = keys.just_pressed(MOVE_KEY);
            let is_copy = keys.just_pressed(COPY_KEY);
            if is_move || is_copy {
                world
                    .save(ToolsPreviewPipeline)
                    .expect("Failed to save copy");
                world
                    .load(ToolsPreviewPipeline)
                    .expect("Failed to load copy");
                let mut q_transform = world.query::<&mut Transform>();
                let mut q_cursor = world
                    .query_filtered::<(&GlobalTransform, Option<&Children>), With<SchematicCursor>>(
                    );

                let (cursor_gt, Some(cchildren)) = q_cursor.single(&world) else {
                    return;
                };
                let offset = cursor_gt.translation();
                let mut children = vec![];
                for c in cchildren {
                    children.push(*c);
                }

                for c in children {
                    let mut t = q_transform.get_mut(world, c).unwrap();
                    (*t).translation = t.translation - offset;
                }

                let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
                next_toolst.set(SchematicToolState::Transform);
                let mut next_transst = world.resource_mut::<NextState<TransformType>>();
                next_transst.set(if is_move {
                    TransformType::Move
                } else {
                    TransformType::Copy
                });
            } else if keys.just_released(WIRE_TOOL_KEY) {
                let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
                next_toolst.set(SchematicToolState::Wiring);
            } else if keys.just_released(KeyCode::KeyR) {
                // spawn a resistor device as child of cursor
                let _ = world.send_event(NewDevice::R);

                let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
                next_toolst.set(SchematicToolState::Transform);
                let mut next_transst = world.resource_mut::<NextState<TransformType>>();
                next_transst.set(TransformType::Copy);
            } else if keys.just_released(KeyCode::KeyA) {
                let mut q_valid = world.query_filtered::<Entity, With<SchematicElement>>();
                let valids = q_valid
                    .iter(&world)
                    .map(|e| (e, Selected))
                    .collect::<Vec<(Entity, Selected)>>();
                let _ = world.insert_or_spawn_batch(valids.into_iter());
            }
        }
        SchematicToolState::Wiring => {}
        SchematicToolState::Transform => {}
    }
}
struct ToolsPreviewPipeline;

impl Pipeline for ToolsPreviewPipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "out/tools"
    }

    /// capture all schematicElements - delete non-selected elements later
    /// this way we circumvent weird entity mapper bug
    fn capture(builder: SnapshotBuilder) -> Snapshot {
        builder
            .deny::<Mesh2dHandle>()
            .deny::<Handle<SchematicMaterial>>()
            .deny::<Handle<DeviceType>>()
            .extract_entities_matching(|e| e.contains::<SchematicElement>())
            .build()
    }

    /// spawning preview copies for previewing tools effect
    /// spawn as children of cursor
    fn apply(world: &mut World, snapshot: &Snapshot) -> Result<(), bevy_save::Error> {
        let mesh_dot = Mesh2dHandle(world.resource::<ElementsRes>().mesh_dot.clone());
        let mesh_unitx = Mesh2dHandle(world.resource::<ElementsRes>().mesh_unitx.clone());
        let mesh_res = Mesh2dHandle(world.resource::<ElementsRes>().mesh_res.clone());
        let mat = world.resource::<ElementsRes>().mat_dflt.clone();
        let sels = world.resource::<ElementsRes>().se_lineseg.clone();
        let selv = world.resource::<ElementsRes>().se_linevertex.clone();
        let sedevice = world.resource::<ElementsRes>().se_device.clone();
        let cursor_ent = world
            .query_filtered::<Entity, With<SchematicCursor>>()
            .single(&world);
        snapshot
            .applier(world)
            .hook(move |entityref, cmd| {
                if entityref.contains::<LineVertex>() {
                    cmd.insert((mesh_dot.clone(), mat.clone(), selv.clone(), Preview));
                }
                if entityref.contains::<LineSegment>() {
                    cmd.insert((mesh_unitx.clone(), mat.clone(), sels.clone(), Preview));
                }
                if entityref.contains::<Device>() {
                    cmd.insert((mesh_res.clone(), mat.clone(), sedevice.clone(), Preview));
                }
                cmd.set_parent(cursor_ent);
            })
            .apply()
    }
}

/*
Update
    input handling (tools)

- pruning?
*/
