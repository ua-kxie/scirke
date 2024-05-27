use bevy::{prelude::*, sprite::Material2dPlugin};

use self::{background::ClipMaterial, cursor::CursorPlugin, grid::GridPlugin};

use super::{camera::SchematicCamera, SnapSet};

mod background;
mod cursor;
mod grid;
mod origin_marker;

pub use cursor::{NewSnappedCursor, SchematicCursor};

pub struct GuidesPlugin;

impl Plugin for GuidesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CursorPlugin);
        app.add_plugins(GridPlugin);
        app.add_plugins(Material2dPlugin::<ClipMaterial>::default());
        // app.add_systems(Startup, origin_marker::setup);
        app.add_systems(Startup, (background::setup, origin_marker::setup));
        app.configure_sets(
            PostUpdate,
            SnapSet.before(bevy::transform::TransformSystem::TransformPropagate),
        );
        app.add_systems(
            PostUpdate,
            (
                origin_marker::main.in_set(SnapSet),
                revert_zoom_scale.in_set(SnapSet),
            ),
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct ZoomInvariant;

/// system is used to set entity scale such that mesh always appear as same size on screen
/// only needs to run when projection scale changes
fn revert_zoom_scale(
    ce: Query<(&OrthographicProjection, Ref<OrthographicProjection>), With<SchematicCamera>>,
    mut qt: ParamSet<(
        Query<&mut Transform, With<ZoomInvariant>>,
        Query<&mut Transform, Changed<ZoomInvariant>>,
    )>,
) {
    let (proj, change) = ce.single();
    match change.is_changed() {
        true => {
            // revert for all tagged entities if projection changed
            for mut t in qt.p0().iter_mut() {
                *t = t.with_scale(Vec3::splat(proj.scale));
            }
        }
        false => {
            // revert for entities newly tagged with zoom invariant
            for mut t in qt.p1().iter_mut() {
                *t = t.with_scale(Vec3::splat(proj.scale));
            }
        }
    }
}
