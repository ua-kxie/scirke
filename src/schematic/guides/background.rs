/*
schematic background
use of single triangle justified here:
https://stackoverflow.com/questions/2588875/whats-the-best-way-to-draw-a-fullscreen-quad-in-opengl-3-2
*/

use bevy::{
    math::vec3,
    prelude::*,
    render::{
        mesh::{Indices::U16, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, ShaderRef},
        view::NoFrustumCulling,
    },
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
};

const Z_DEPTH: f32 = -0.9;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut clip_materials: ResMut<Assets<ClipMaterial>>,
) {
    // spawn clip space background
    // use of single triangle justified here:
    // https://stackoverflow.com/questions/2588875/whats-the-best-way-to-draw-a-fullscreen-quad-in-opengl-3-2
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            vec3(1.0, 3.0, 0.0),
            vec3(1.0, -1.0, 0.0),
            vec3(-3.0, -1.0, 0.0),
        ],
    )
    .with_inserted_indices(U16(vec![0, 1, 2]));

    let meshid = meshes.add(mesh);
    let bundle = (
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshid),
            material: clip_materials.add(ClipMaterial { color: Color::RED }),
            transform: Transform::from_translation(Vec3::Z * Z_DEPTH),
            ..Default::default()
        },
        NoFrustumCulling,
    );
    commands.spawn(bundle);
}

/// clip space material: vertex shader applies a custom uniform transform to vertices
/// used to draw background - although current impl probably costs more than it saves
/// compared to using an existing pipeline
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ClipMaterial {
    #[uniform(0)]
    pub color: Color,
}

impl Material2d for ClipMaterial {
    fn vertex_shader() -> ShaderRef {
        "clipspace.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "clipspace.wgsl".into()
    }
}
