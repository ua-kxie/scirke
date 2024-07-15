use bevy::{prelude::*, sprite::Mesh2dHandle, text::TextLayoutInfo};
use bevy_save::prelude::*;

mod devicespawn;
mod sel;
mod transform;
mod wire;

use super::{
    electrical::{Preview, SchematicElement},
    guides::SchematicCursor,
    material::SchematicMaterial,
    FreshLoad,
};
pub use sel::{NewPickingCollider, PickingCollider, SelectEvt};

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
        app.init_state::<SchematicToolState>();
        app.add_event::<MergeLoadEvent>();
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
