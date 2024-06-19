use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_save::prelude::*;
use transform::TransformType;

mod sel;
mod transform;
mod wire;

use super::{
    electrical::{DefaultDevices, PickableElement, Preview, SchematicElement, Selected},
    guides::SchematicCursor,
    material::SchematicMaterial,
    FreshLoad, SchematicLoaded,
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
        app.init_state::<SchematicToolState>();
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
            let newg = keys.just_pressed(KeyCode::KeyG);
            let newvs = keys.just_pressed(KeyCode::KeyV);
            let newis = keys.just_pressed(KeyCode::KeyI);
            let newr = keys.just_pressed(KeyCode::KeyR);
            let newl = keys.just_pressed(KeyCode::KeyL);
            let newc = keys.just_pressed(KeyCode::KeyK);

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
                world.send_event(SchematicLoaded);
            } else if keys.just_released(WIRE_TOOL_KEY) {
                let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
                next_toolst.set(SchematicToolState::Wiring);
            } else if newg || newvs || newis || newr || newl || newc {
                // spawn a resistor device as child of cursor
                let dd = world.get_resource::<DefaultDevices>().unwrap();
                let _ = world.send_event(if newvs {
                    dd.voltage_source()
                } else if newis {
                    dd.current_source()
                } else if newr {
                    dd.resistor()
                } else if newl {
                    dd.inductor()
                } else if newc {
                    dd.capacitor()
                } else {
                    dd.gnd()
                });

                let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
                next_toolst.set(SchematicToolState::Transform);
                let mut next_transst = world.resource_mut::<NextState<TransformType>>();
                next_transst.set(TransformType::Copy);
            } else if keys.just_released(KeyCode::KeyA) {
                let mut q_valid = world.query_filtered::<Entity, With<PickableElement>>();
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
            // .deny::<SpId>() // should be kept for move, denied for copy
            // .deny::<NetId>() // should be kept for move, denied for copy
            .extract_entities_matching(|e| e.contains::<SchematicElement>())
            .build()
    }

    /// spawning preview copies for previewing tools effect
    /// spawn as children of cursor
    fn apply(world: &mut World, snapshot: &Snapshot) -> Result<(), bevy_save::Error> {
        let cursor_ent = world
            .query_filtered::<Entity, With<SchematicCursor>>()
            .single(&world);
        snapshot
            .applier(world)
            .hook(move |_entityref, cmd| {
                cmd.insert((Preview, FreshLoad));
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
