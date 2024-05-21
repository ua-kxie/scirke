/*
schematic background
*/

use bevy::{
    math::vec3,
    prelude::*,
    render::{
        mesh::{Indices::U16, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        view::NoFrustumCulling,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::bevyon::ClipMaterial;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut clip_materials: ResMut<Assets<ClipMaterial>>,
) {
    // spawn clip space background
    // use of single triangle justified here:
    // https://stackoverflow.com/questions/2588875/whats-the-best-way-to-draw-a-fullscreen-quad-in-opengl-3-2
    let far_plane_depth = 1.0;
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
    let bundle = (
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshid),
            material: clip_materials.add(ClipMaterial { color: Color::TEAL }),
            ..Default::default()
        },
        NoFrustumCulling,
    );
    commands.spawn(bundle);
}
