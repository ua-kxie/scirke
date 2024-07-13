use bevy::{input::{keyboard::KeyboardInput, ButtonState}, prelude::*, sprite::Mesh2dHandle, text::TextLayoutInfo};
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
        app.add_systems(PreUpdate, (
            tools_select.in_set(EntityLoadSet::Direct),
            (save_load, post_serde).chain().run_if(on_event::<MergeLoadEvent>()),
        ).chain());
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
    mut ev_sch_changed: EventWriter<SchematicChanged>,
) {
    let cursor = qc.single();
    // despawn Preview, Pickable, Without<Selected>
    commands
        .entity(cursor)
        .remove_children(&q_unpicked.iter().collect::<Box<[Entity]>>());
    for e in q_unpicked.iter() {
        debug!("deletied entity: unpicked, selected");
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
    ev_sch_changed.send(SchematicChanged); // TODO port gets double dipped between transform propagate and port location update
}

fn save_load(world: &mut World) {
    debug!("save-load");
    // everything in schematic gets saved
    world
        .save(ToolsPreviewPipeline)
        .expect("Failed to save copy");
    // copy of everything gets loaded with Preview tag
    world
        .load(ToolsPreviewPipeline)
        .expect("Failed to load copy");
}

fn tools_select(
    mut commands: Commands,
    mut evt_keys: EventReader<KeyboardInput>,
    toolst_now: ResMut<State<SchematicToolState>>,
    mut toolst_next: ResMut<NextState<SchematicToolState>>,
    mut transformst_next: ResMut<NextState<TransformType>>,
    q_sel: Query<Entity, With<Selected>>,
    q_valid_sel: Query<Entity, With<PickableElement>>,
    mut evtw_mergeload: EventWriter<MergeLoadEvent>,
    mut evtw_load: EventWriter<LoadEvent>,
) {
    let evt_keys = evt_keys.read().collect::<Vec<&KeyboardInput>>();
    if evt_keys.iter().any(|ki| ki.key_code == KeyCode::Escape && ki.state == ButtonState::Released) {
        toolst_next.set(SchematicToolState::Idle);
        return
    }
    match toolst_now.get() {
        SchematicToolState::Idle => {
            let is_move = evt_keys.iter().any(|ki| ki.key_code == MOVE_KEY);
            let is_copy = evt_keys.iter().any(|ki| ki.key_code == COPY_KEY);
            if is_move || is_copy {
                // check if valid (something selected)
                if !q_sel.is_empty() {
                    debug!("selecting transform tool");
                    toolst_next.set(SchematicToolState::Transform);
                    transformst_next.set(if is_move {
                        debug!("selecting transform::move");
                        TransformType::Move
                    } else {
                        debug!("selecting transform::copy");
                        TransformType::Copy
                    });
                    evtw_mergeload.send(MergeLoadEvent);
                    evtw_load.send(LoadEvent);
                }
            } else if evt_keys.iter().any(|ki| ki.key_code == WIRE_TOOL_KEY && ki.state == ButtonState::Released) {
                toolst_next.set(SchematicToolState::Wiring);
            } else if evt_keys.iter().any(|ki| ki.key_code == DEVICE_SPAWN_TOOL_KEY && ki.state == ButtonState::Released) {
                debug!("selecting device spawn tool");
                toolst_next.set(SchematicToolState::DeviceSpawn);
            } else if evt_keys.iter().any(|ki| ki.key_code == KeyCode::KeyA && ki.state == ButtonState::Released) {
                // select all
                let valids = q_valid_sel
                    .iter()
                    .map(|e| (e, Selected))
                    .collect::<Vec<(Entity, Selected)>>();
                commands.insert_or_spawn_batch(valids.into_iter());
            }
        },
        SchematicToolState::Wiring => {},
        SchematicToolState::Transform => {},
        SchematicToolState::DeviceSpawn => {},
    }
}

// fn exclusive_main(world: &mut World) {
//     // let keys = world.resource::<ButtonInput<KeyCode>>();
//     let keys = world.resource::<ResMut<EventReader<KeyboardInput>>>();
//     let curr_toolst = world.resource::<State<SchematicToolState>>();

//     if keys.just_released(KeyCode::Escape) {
//         let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
//         next_toolst.set(SchematicToolState::Idle);
//         return;
//     }
//     match curr_toolst.get() {
//         SchematicToolState::Idle => {
//             let is_move = keys.just_pressed(MOVE_KEY);
//             let is_copy = keys.just_pressed(COPY_KEY);
//             if is_move || is_copy {
//                 // check if valid (something selected)
//                 let mut q_anything_selected = world.query_filtered::<Entity, With<Selected>>();
//                 let mut something_selected = false;
//                 for _ in q_anything_selected.iter(&world) {
//                     // valid
//                     something_selected = true;
//                     break;
//                 }
//                 if something_selected {
//                     // everything in schematic gets saved
//                     world
//                         .save(ToolsPreviewPipeline)
//                         .expect("Failed to save copy");
//                     // copy of everything gets loaded with Preview tag
//                     world
//                         .load(ToolsPreviewPipeline)
//                         .expect("Failed to load copy");
//                     let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
//                     next_toolst.set(SchematicToolState::Transform);
//                     let mut next_transst = world.resource_mut::<NextState<TransformType>>();
//                     next_transst.set(if is_move {
//                         TransformType::Move
//                     } else {
//                         TransformType::Copy
//                     });
//                     world.send_event(MergeLoadEvent);
//                     world.send_event(LoadEvent);
//                 }
//             } else if keys.just_released(WIRE_TOOL_KEY) {
//                 let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
//                 next_toolst.set(SchematicToolState::Wiring);
//             } else if keys.just_released(DEVICE_SPAWN_TOOL_KEY) {
//                 let mut next_toolst = world.resource_mut::<NextState<SchematicToolState>>();
//                 next_toolst.set(SchematicToolState::DeviceSpawn);
//             } else if keys.just_released(KeyCode::KeyA) {
//                 let mut q_valid = world.query_filtered::<Entity, With<PickableElement>>();
//                 let valids = q_valid
//                     .iter(&world)
//                     .map(|e| (e, Selected))
//                     .collect::<Vec<(Entity, Selected)>>();
//                 let _ = world.insert_or_spawn_batch(valids.into_iter());
//             }
//         }
//         SchematicToolState::Wiring => {}
//         SchematicToolState::Transform => {}
//         SchematicToolState::DeviceSpawn => {}
//     }
// }
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
