use crate::bevyon::ClipMaterial;

use self::{
    camera::{CameraPlugin, SchematicCamera},
    guides::GuidesPlugin,
    infotext::InfoPlugin,
};
use bevy::{prelude::*, sprite::Material2dPlugin};

mod camera;
mod guides;
mod infotext;

// Snapped marker component: system to goes around snapping transform of such entities
#[derive(Component)]
pub struct Snap {
    /// snap step size, coords are snapped as:
    /// (coord/step).round() * step
    pub world_step: f32,
    // pub space: Space,
}

impl Snap {
    const DEFAULT: Self = Snap {
        world_step: 1.0,
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
        app.add_plugins((CameraPlugin, InfoPlugin, GuidesPlugin));
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
) {
    for (mut t, s) in e.iter_mut() {
        t.translation = (t.translation / s.world_step).round() * s.world_step;
    }
}
