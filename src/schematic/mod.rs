use crate::bevyon::ClipMaterial;

use self::{camera::CameraPlugin, cursor::CursorPlugin, infotext::InfoPlugin};
use bevy::{
    math::vec3,
    prelude::*,
    render::{
        mesh::{Indices::U16, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

mod camera;
mod cursor;
mod infotext;

// Snapped marker component: system to goes around snapping transform of such entities
#[derive(Component)]
pub struct Snapped {
    /// snap step size, coords are snapped as:
    /// (coord/step).round() * step
    pub step: f32,
}

impl Snapped {
    const DEFAULT: Self = Snapped { step: 1.0 };
}

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CameraPlugin, CursorPlugin, InfoPlugin));
        app.add_plugins(Material2dPlugin::<ClipMaterial>::default());
        app.add_systems(Startup, setup);
        // app.add_systems(
        //     Update,
        //     (
        //         camera_transform,
        //         cursor_update,
        //         draw_curpos_ssp,
        //         visible_canvas_aabb,
        //         update_info_text,
        //     ),
        // );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut clip_materials: ResMut<Assets<ClipMaterial>>,
) {
    // spawn some marker object for testing
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        transform: Transform::default()
            .with_scale(Vec3::splat(1.))
            .with_translation(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.1,
            }),
        material: materials.add(Color::PURPLE),
        ..default()
    });

    // spawn clip space background
    // use of single triangle justified here:
    // https://stackoverflow.com/questions/2588875/whats-the-best-way-to-draw-a-fullscreen-quad-in-opengl-3-2
    let far_plane_depth = 0.0;
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            vec3(1.0, 3.0, far_plane_depth),
            vec3(1.0, -1.0, far_plane_depth),
            vec3(-3.0, -1.0, far_plane_depth),
        ],
    )
    .with_inserted_indices(U16(vec![0, 1, 2]));

    let meshid = meshes.add(mesh);
    let bundle = MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshid),
        material: clip_materials.add(ClipMaterial {
            z_depth: far_plane_depth,
            color: Color::GRAY,
        }),
        ..Default::default()
    };
    commands.spawn(bundle);
}
