use self::{
    camera::CameraPlugin, elements::ElementsPlugin, guides::GuidesPlugin, infotext::InfoPlugin, material::{SchematicMaterial, WireMaterial}, tools::ToolsPlugin
};
use bevy::{prelude::*, render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages}, sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle}};

mod camera;
mod guides;
mod infotext;
mod material;
mod elements;
mod tools;

// Snapped marker component: system to goes around snapping transform of such entities
#[derive(Component)]
pub struct Snap {
    /// snap step size, coords are snapped as:
    /// (coord/step).round() * step
    pub world_step: f32,
    // pub space: Space,
}

impl Snap {
    const DEFAULT: Self = Snap { world_step: 1.0 };
}

pub enum Space {
    World,
    Clip,
}

/// [`SystemSet`] for system which performs snapping.
/// Resides in [`PostUpdate`] schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SnapSet;

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CameraPlugin, InfoPlugin, GuidesPlugin, ElementsPlugin, ToolsPlugin));
        app.configure_sets(
            PostUpdate,
            SnapSet.before(bevy::transform::TransformSystem::TransformPropagate),
        );
        app.add_systems(Startup, setup);
        app.add_systems(PostUpdate, snap.in_set(SnapSet));
        app.add_plugins(Material2dPlugin::<SchematicMaterial>::default());
        app.add_plugins(Material2dPlugin::<WireMaterial>::default());
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
    mut materials: ResMut<Assets<WireMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mat_bundle = MaterialMesh2dBundle {
        // TODO: automatic batching need instances to share the same mesh
        mesh: Mesh2dHandle(meshes.add(
            Mesh::new(
                PrimitiveTopology::LineList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::ZERO, Vec3::X])
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![Vec4::ONE, Vec4::ONE])
            .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1]))
        )),
        material: materials.add(WireMaterial {
            color: Color::WHITE,
        }),
        transform: Transform::from_scale(Vec3::splat(1.0)),
        ..Default::default()
    };
    commands.spawn(mat_bundle);
}
