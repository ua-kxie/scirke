use bevy::{prelude::*, sprite::Mesh2dHandle, text::TextLayoutInfo};
use bevy_save::prelude::*;
use transform::TransformType;

mod devicespawn;
mod sel;
mod transform;
mod wire;

use super::{
    electrical::{PickableElement, Preview, SchematicElement, Selected},
    guides::SchematicCursor,
    material::SchematicMaterial,
    EntityLoadSet, FreshLoad, LoadEvent, SchematicChanged,
};
pub use sel::{NewPickingCollider, PickingCollider, SelectEvt};

const WIRE_TOOL_KEY: KeyCode = KeyCode::KeyW;
const DEVICE_SPAWN_TOOL_KEY: KeyCode = KeyCode::KeyD;
const MOVE_KEY: KeyCode = KeyCode::KeyM;
const COPY_KEY: KeyCode = KeyCode::KeyC;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum SchematicToolState {
    #[default]
    Idle, // also select
    Wiring,    // for drawing wires
    Transform, // moving elements around,
    DeviceSpawn, // for spawning a new device,
               // Label,   // wire/net labeling
               // Comment, // plain text comment with basic formatting options
}

#[derive(Event)]
pub struct MergeLoadEvent;

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            wire::WireToolPlugin,
            sel::SelToolPlugin,
            transform::TransformToolPlugin,
            devicespawn::DeviceSpawnToolPlugin,
        ));
        app.add_systems(PreUpdate, exclusive_main.in_set(EntityLoadSet::Direct));
        app.add_systems(
            PreUpdate,
            post_serde
                .in_set(EntityLoadSet::Post)
                .run_if(on_event::<MergeLoadEvent>()),
        );
        app.init_state::<SchematicToolState>();
        app.add_event::<MergeLoadEvent>();
    }
}

fn post_serde(
    mut commands: Commands,
    qc: Query<Entity, With<SchematicCursor>>,
    q_unpicked: Query<Entity, (Without<Selected>, With<Preview>, With<PickableElement>)>,
    mut q_transform: Query<(&GlobalTransform, &mut Transform)>,
    q_scparent: Query<(&GlobalTransform, &Children), With<SchematicCursor>>,
) {
    let cursor = qc.single();
    // despawn Preview, Pickable, Without<Selected>
    commands
        .entity(cursor)
        .remove_children(&q_unpicked.iter().collect::<Box<[Entity]>>());
    for e in q_unpicked.iter() {
        dbg!("8");
        commands.entity(e).despawn();
    }
    // anything Preview gets added to schematiccursor children, transform adjusted
    let (cursor_gt, cchildren) = q_scparent.single();
    let offset = cursor_gt.translation();
    let mut children = vec![];
    for c in cchildren {
        children.push(*c);
    }
    for c in children {
        let (gt, mut t) = q_transform.get_mut(c).unwrap();
        (*t).translation = gt.translation() - offset;
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
                // check if valid (something selected)
                let mut q_anything_selected = world.query_filtered::<Entity, With<Selected>>();
                let mut something_selected = false;
                for _ in q_anything_selected.iter(&world) {
                    // valid
                    something_selected = true;
                    break;
                }
                if something_selected {
                    // everything in schematic gets saved
                    world
                        .save(ToolsPreviewPipeline)
                        .expect("Failed to save copy");
                    // copy of everything gets loaded with Preview tag
                    world
                        .load(ToolsPreviewPipeline)
                        .expect("Failed to load copy");
                    let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
                    next_toolst.set(SchematicToolState::Transform);
                    let mut next_transst = world.resource_mut::<NextState<TransformType>>();
                    next_transst.set(if is_move {
                        TransformType::Move
                    } else {
                        TransformType::Copy
                    });
                    world.send_event(MergeLoadEvent);
                    world.send_event(LoadEvent);
                }
            } else if keys.just_released(WIRE_TOOL_KEY) {
                let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
                next_toolst.set(SchematicToolState::Wiring);
            } else if keys.just_released(DEVICE_SPAWN_TOOL_KEY) {
                let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
                next_toolst.set(SchematicToolState::DeviceSpawn);
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
        SchematicToolState::DeviceSpawn => {}
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
            .deny::<TextLayoutInfo>() // TODO: should work without this with bevy 0.14
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
