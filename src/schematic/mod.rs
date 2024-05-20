use crate::bevyon::ClipMaterial;

use self::{
    camera::{CameraPlugin, SchematicCamera}, cursor::CursorPlugin, guides::GuidesPlugin, infotext::InfoPlugin
};
use bevy::{
    math::vec3,
    prelude::*,
    render::{
        mesh::{Indices::U16, PrimitiveTopology},
        render_asset::RenderAssetUsages, view::NoFrustumCulling,
    },
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

mod camera;
mod cursor;
mod infotext;
mod guides;

// Snapped marker component: system to goes around snapping transform of such entities
#[derive(Component)]
pub struct Snap {
    /// snap step size, coords are snapped as:
    /// (coord/step).round() * step
    pub world_step: f32,
    pub space: Space,
}

impl Snap {
    const DEFAULT_WORLD: Self = Snap {
        world_step: 1.0,
        space: Space::World,
    };
    const DEFAULT_CLIP: Self = Snap {
        world_step: 1.0,
        space: Space::Clip,
    };
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
        app.add_plugins((CameraPlugin, CursorPlugin, InfoPlugin, GuidesPlugin));
        app.add_plugins(Material2dPlugin::<ClipMaterial>::default());
        app.configure_sets(
            PostUpdate,
            SnapSet.before(bevy::transform::TransformSystem::TransformPropagate),
        );
        app.add_systems(PostUpdate, snap.in_set(SnapSet));
    }
}

/// this system snaps all applicable entities
fn snap(
    mut e: Query<(&mut Transform, &Snap), Changed<GlobalTransform>>,
    c: Query<(&Camera, &GlobalTransform), With<SchematicCamera>>,
) {
    let (cam, cgt) = c.single();
    for (mut t, s) in e.iter_mut() {
        match s.space {
            Space::World => t.translation = (t.translation / s.world_step).round() * s.world_step,
            Space::Clip => {
                let world_coords = cam.ndc_to_world(cgt, t.translation).unwrap();
                let world_coords = (world_coords / s.world_step).round() * s.world_step;
                t.translation = cam.world_to_ndc(cgt, world_coords).unwrap();
            }
        }
    }
}
