use self::{
    camera::CameraPlugin, elements::ElementsPlugin, guides::GuidesPlugin, infotext::InfoPlugin,
    material::SchematicMaterial, tools::ToolsPlugin,
};
use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_save::prelude::*;
use elements::{lsse, lvse, ElementsRes, LineSegment, LineVertex, SchematicElement};

mod camera;
mod elements;
mod guides;
mod infotext;
mod material;
mod tools;

// Snapped marker component: system to goes around snapping transform of such entities
#[derive(Component)]
pub struct Snap {
    pub world_step: f32,
}

impl Snap {
    const DEFAULT: Self = Snap { world_step: 1.0 };
}

/// [`SystemSet`] for system which performs snapping.
/// Resides in [`PostUpdate`] schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SnapSet;

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
        ));
        app.configure_sets(
            PostUpdate,
            SnapSet.before(bevy::transform::TransformSystem::TransformPropagate),
        );
        app.add_systems(PostUpdate, snap.in_set(SnapSet));
        app.add_systems(Update, handle_save_input);
        app.add_plugins((
            // Bevy Save
            SavePlugins,
        ));
    }
}

/// this system snaps all applicable entities
fn snap(mut e: Query<(&mut Transform, &Snap), Changed<GlobalTransform>>) {
    for (mut t, s) in e.iter_mut() {
        t.translation = (t.translation / s.world_step).round() * s.world_step;
    }
}

/// helper system to test things quick and dirty
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<SchematicMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mat_bundle = MaterialMesh2dBundle {
        mesh: Mesh2dHandle(
            meshes.add(
                Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO, Vec3::X])
                    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![Vec4::ONE, Vec4::ONE])
                    .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1])),
            ),
        ),
        material: materials.add(SchematicMaterial {
            color: Color::BLACK.with_a(0.0),
        }),
        transform: Transform::from_scale(Vec3::splat(1.0)),
        ..Default::default()
    };
    commands.spawn(mat_bundle);
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
        let mesh_dot = Mesh2dHandle(world.resource::<ElementsRes>().mesh_dot.clone());
        let mesh_unitx = Mesh2dHandle(world.resource::<ElementsRes>().mesh_unitx.clone());
        let mat = world.resource::<ElementsRes>().mat_dflt.clone();
        snapshot
            .applier(world)
            .despawn::<With<SchematicElement>>()
            .hook(move |entity, cmd| {
                if entity.contains::<LineVertex>() {
                    cmd.insert((mesh_dot.clone(), mat.clone(), lvse()));
                }
                if entity.contains::<LineSegment>() {
                    cmd.insert((mesh_unitx.clone(), mat.clone(), lsse()));
                }
            })
            .apply()
    }
}

fn handle_save_input(world: &mut World) {
    let keys = world.resource::<ButtonInput<KeyCode>>();

    if keys.just_released(KeyCode::Space) {
        world.checkpoint::<SavePipeline>();
    } else if keys.just_released(KeyCode::Enter) {
        world.save(SavePipeline).expect("Failed to save");
    } else if keys.just_released(KeyCode::Backspace) {
        world.load(SavePipeline).expect("Failed to load");
    } else if keys.just_released(KeyCode::KeyA) {
        world
            .rollback::<SavePipeline>(1)
            .expect("Failed to rollback");
    } else if keys.just_released(KeyCode::KeyD) {
        world
            .rollback::<SavePipeline>(-1)
            .expect("Failed to rollforward");
    }
}
