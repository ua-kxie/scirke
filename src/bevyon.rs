/* plugin to help run lyon as tessellator */

use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, ShaderRef},
    },
    sprite::{Material2d, Mesh2dHandle},
};

pub use lyon_tessellation::{self as tess};

use bevy::render::mesh::Indices::U32;

use tess::{
    path::{builder::NoAttributes, BuilderImpl},
    BuffersBuilder, FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor,
};
pub use tess::{FillOptions, StrokeOptions};

/*
helper stuff not directly tied to bevy
*/
/// u32: The index type of a Bevy [`Mesh`](bevy::render::mesh::Mesh).
type VertexBuffers = tess::VertexBuffers<Vec2, u32>;

pub fn path_builder() -> NoAttributes<BuilderImpl> {
    tess::path::path::Builder::new()
}

/// Zero-sized type used to implement various vertex construction traits from
/// Lyon.
pub struct VertexConstructor;

/// Enables the construction of a [`Vertex`] when using a `FillTessellator`.
impl FillVertexConstructor<Vec2> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vec2 {
        Vec2 {
            x: vertex.position().x,
            y: vertex.position().y,
        }
    }
}

/// Enables the construction of a [`Vertex`] when using a `StrokeTessellator`.
impl StrokeVertexConstructor<Vec2> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vec2 {
        Vec2 {
            x: vertex.position().x,
            y: vertex.position().y,
        }
    }
}
/*
bevy tied stuff
*/
#[derive(Resource, Deref, DerefMut)]
struct FillTessellator(lyon_tessellation::FillTessellator);

#[derive(Resource, Deref, DerefMut)]
struct StrokeTessellator(lyon_tessellation::StrokeTessellator);

/// Tessellator Input Data
/// queries for Changed<>
#[derive(Component, Default)]
pub struct TessInData {
    pub path: Option<tess::path::Path>,
    pub stroke: Option<tess::StrokeOptions>,
    pub fill: Option<tess::FillOptions>,
    pub z_depth: f32,
}

pub struct BevyonPlugin;

impl Plugin for BevyonPlugin {
    fn build(&self, app: &mut App) {
        let fill_tess = tess::FillTessellator::new();
        let stroke_tess = tess::StrokeTessellator::new();
        app.insert_resource(FillTessellator(fill_tess))
            .insert_resource(StrokeTessellator(stroke_tess));
        app.configure_sets(
            PostUpdate,
            BuildShapes.after(bevy::transform::TransformSystem::TransformPropagate),
        )
        .add_systems(PostUpdate, update_mesh.in_set(BuildShapes));
    }
}

/// [`SystemSet`] for the system that builds the meshes for newly-added
/// or changed shapes. Resides in [`PostUpdate`] schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct BuildShapes;

fn update_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    mut fill_tess: ResMut<FillTessellator>,
    mut stroke_tess: ResMut<StrokeTessellator>,
    mut query: Query<(&TessInData, &mut Mesh2dHandle), Changed<TessInData>>,
) {
    for (data, mut mesh) in &mut query {
        let mut buffers = VertexBuffers::new();
        if let Some(path) = &data.path {
            if let Some(options) = data.fill {
                fill(&mut fill_tess, &path, &options, &mut buffers);
            }
            if let Some(options) = data.stroke {
                stroke(&mut stroke_tess, &path, &options, &mut buffers);
            }
        }
        mesh.0 = meshes.add(build_mesh(&buffers, data.z_depth));
    }
}

fn fill(
    tess: &mut ResMut<FillTessellator>,
    path: &tess::path::Path,
    options: &FillOptions,
    buffers: &mut VertexBuffers,
) {
    if let Err(e) = tess.tessellate_path(
        path,
        &options,
        &mut BuffersBuilder::new(buffers, VertexConstructor),
    ) {
        error!("FillTessellator error: {:?}", e);
    }
}

fn stroke(
    tess: &mut ResMut<StrokeTessellator>,
    path: &tess::path::Path,
    options: &StrokeOptions,
    buffers: &mut VertexBuffers,
) {
    if let Err(e) = tess.tessellate_path(
        path,
        &options,
        &mut BuffersBuilder::new(buffers, VertexConstructor),
    ) {
        error!("StrokeTessellator error: {:?}", e);
    }
}

fn build_mesh(buffers: &VertexBuffers, z_depth: f32) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_indices(U32(buffers.indices.clone()));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        buffers
            .vertices
            .iter()
            .map(|v| [v.x, v.y, z_depth])
            .collect::<Vec<[f32; 3]>>(),
    );

    mesh
}

/// clip space material: vertex shader applies a custom uniform transform to vertices
///
/// skipping vertex shader nullifies geometry scaling - desired for cursor, grid dots, and maybe others
/// drawing relative to camera is insufficient, since scaling/fov will change apparent size of geometry.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ClipMaterial {
    pub z_depth: f32,
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
    fn depth_bias(&self) -> f32 {
        self.z_depth
    }
}
