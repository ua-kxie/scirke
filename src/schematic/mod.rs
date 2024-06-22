use self::{
    camera::CameraPlugin, electrical::ElementsPlugin, guides::GuidesPlugin, infotext::InfoPlugin,
    material::SchematicMaterial, tools::ToolsPlugin,
};
use bevy::{
    prelude::*,
    sprite::{Material2dPlugin, Mesh2dHandle},
};
use bevy_save::prelude::*;
use electrical::SchematicElement;
use ui::SchematicUiPlugin;

mod camera;
mod electrical;
mod guides;
mod infotext;
mod material;
mod tools;
mod ui;
// Snapped marker component: system to goes around snapping transform of such entities
#[derive(Component)]
pub struct Snap {
    pub world_step: f32,
}

impl Snap {
    const DEFAULT: Self = Snap { world_step: 1.0 };
}

/// event to fire whenever the schematic change in a way that should be checkpointed
#[derive(Event)]
pub struct SchematicChanged;

/// [`SystemSet`] for system which performs snapping.
/// Resides in [`PostUpdate`] schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SnapSet;

/// SystemSet for deserializing
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum EntityLoadSet {
    /// systems doing serde
    Direct,

    /// systems which react to a schematic with freshly loaded entities, add mesh handle, PickableElement, etc.
    React,

    /// systems which need to work with a world that has otherwise finished deserializing
    Post,
}

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<SchematicMaterial>::default());
        app.add_plugins((
            CameraPlugin,
            InfoPlugin,
            GuidesPlugin,
            ElementsPlugin,
            ToolsPlugin,
            SchematicUiPlugin,
        ));
        app.configure_sets(
            PostUpdate,
            SnapSet.before(bevy::transform::TransformSystem::TransformPropagate),
        );
        app.add_systems(Update, snap.in_set(SnapSet));
        app.add_systems(PostStartup, register_checkpoint); // so we can rollback to initial state
        app.add_systems(
            Update,
            register_checkpoint.run_if(on_event::<SchematicChanged>()),
        );
        app.add_systems(PreUpdate, process_checkpoints.in_set(EntityLoadSet::Direct));
        app.add_plugins((
            // Bevy Save
            SavePlugins,
        ));
        app.configure_sets(
            PreUpdate,
            (
                EntityLoadSet::Direct,
                EntityLoadSet::React
                    .run_if(on_event::<LoadEvent>())
                    .after(EntityLoadSet::Direct),
                EntityLoadSet::Post
                    .run_if(on_event::<LoadEvent>())
                    .after(EntityLoadSet::React),
            ),
        );
        app.add_event::<SchematicChanged>();
        app.add_event::<LoadEvent>();
    }
}

/// this system snaps all applicable entities
fn snap(mut e: Query<(&mut Transform, &Snap), Changed<GlobalTransform>>) {
    for (mut t, s) in e.iter_mut() {
        t.translation = (t.translation / s.world_step).round() * s.world_step;
    }
}

struct SavePipeline;

impl Pipeline for SavePipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "out/saves"
    }

    fn capture(builder: SnapshotBuilder) -> Snapshot {
        builder
            .deny::<Mesh2dHandle>()
            .deny::<Handle<SchematicMaterial>>()
            .extract_entities_matching(|e| e.contains::<SchematicElement>())
            .extract_rollbacks()
            .build()
    }

    fn apply(world: &mut World, snapshot: &Snapshot) -> Result<(), bevy_save::Error> {
        snapshot
            .applier(world)
            .despawn::<With<SchematicElement>>()
            .hook(move |_entityref, cmd| {
                cmd.insert(FreshLoad);
            })
            .apply()
    }
}

/// saving
/// loading
fn register_checkpoint(world: &mut World) {
    world.checkpoint::<SavePipeline>();
}
fn process_checkpoints(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();
    if keys.just_pressed(KeyCode::KeyZ) && keys.pressed(KeyCode::ControlLeft) {
        if keys.pressed(KeyCode::ShiftLeft) {
            world
                .rollback::<SavePipeline>(-1)
                .expect("Failed to rollforward");
            world.send_event(LoadEvent);
        } else {
            world
                .rollback::<SavePipeline>(1)
                .expect("Failed to rollback");
            world.send_event(LoadEvent);
        }
        // world.send_event(SchematicChanged);
    }

    // if keys.just_released(KeyCode::Space) {
    //     world.checkpoint::<SavePipeline>();
    // } else if keys.just_released(KeyCode::Enter) {
    //     world.save(SavePipeline).expect("Failed to save");
    // } else if keys.just_released(KeyCode::Backspace) {
    //     world.load(SavePipeline).expect("Failed to load");
    // } else if keys.just_released(KeyCode::KeyA) {
    //     world
    //         .rollback::<SavePipeline>(1)
    //         .expect("Failed to rollback");
    // } else if keys.just_released(KeyCode::KeyD) {
    //     world
    //         .rollback::<SavePipeline>(-1)
    //         .expect("Failed to rollforward");
    // }
}

/// event to fire when elements are loaded in through reflection
/// systems should watch for this event and append non-reflect components
#[derive(Event)]
pub struct LoadEvent;

/// component to tag entities that have just been loaded and need to append non-reflect components
#[derive(Component)]
pub struct FreshLoad;
