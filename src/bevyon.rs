/* plugin to help run lyon as tessellator */

use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
    sprite::Mesh2dHandle,
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
#[derive(Resource)]
pub struct EmptyMesh(pub Mesh);

impl Default for EmptyMesh {
    fn default() -> Self {
        Self(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<Vec3>::new())
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<Vec4>::new())
            .with_inserted_indices(U32(vec![])),
        )
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct FillTessellator(lyon_tessellation::FillTessellator);

impl Default for FillTessellator {
    fn default() -> Self {
        Self(tess::FillTessellator::new())
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct StrokeTessellator(lyon_tessellation::StrokeTessellator);

impl Default for StrokeTessellator {
    fn default() -> Self {
        Self(tess::StrokeTessellator::new())
    }
}
/// Tessellator Input Data
#[derive(Default)]
pub struct TessInData {
    pub path: Option<tess::path::Path>,
    pub stroke: Option<tess::StrokeOptions>,
    pub fill: Option<tess::FillOptions>,
}

/// data to build single colored mesh
#[derive(Default)]
pub struct SubMesh {
    pub tess_data: TessInData,
    pub color: Color,
}

impl SubMesh {
    pub fn new_with_color(tess_data: TessInData, color: Color) -> Self {
        SubMesh { tess_data, color }
    }
}

/// bevyon Input Data
/// queries for Changed<>
/// allows for building meshes with different color attributes using mesh merge
#[derive(Default, Component, Deref, DerefMut)]
pub struct CompositeMeshData {
    pub mesh_data: Vec<SubMesh>,
}

impl CompositeMeshData {
    pub fn from_single(single: TessInData) -> Self {
        Self {
            mesh_data: vec![SubMesh {
                tess_data: single,
                color: Color::WHITE,
            }],
        }
    }
    pub fn from_single_w_color(single: TessInData, color: Color) -> Self {
        Self {
            mesh_data: vec![SubMesh {
                tess_data: single,
                color,
            }],
        }
    }
}

pub struct BevyonPlugin;

impl Plugin for BevyonPlugin {
    fn build(&self, app: &mut App) {
        // let fill_tess = tess::FillTessellator::new();
        // let stroke_tess = tess::StrokeTessellator::new();
        app.insert_resource(FillTessellator::default())
            .insert_resource(StrokeTessellator::default())
            .insert_resource(EmptyMesh::default());
        app.configure_sets(
            PostUpdate,
            BuildShapes.after(bevy::transform::TransformSystem::TransformPropagate),
        )
        .add_systems(PostUpdate, update_mesh.in_set(BuildShapes));
        // app.add_systems(PreStartup, setup);
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
    mut query: Query<(&CompositeMeshData, &mut Mesh2dHandle), Changed<CompositeMeshData>>,
    empty_mesh: Res<EmptyMesh>,
) {
    for (data, mut meshndl) in &mut query {
        let empty = empty_mesh.0.clone();
        let mesh = build_mesh_im(empty, data, &mut stroke_tess, &mut fill_tess);
        meshndl.0 = meshes.add(mesh);
    }
}

fn fill(
    tess: &mut FillTessellator,
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
    tess: &mut StrokeTessellator,
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

pub fn build_mesh(buffers: &VertexBuffers) -> Mesh {
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
            .map(|v| [v.x, v.y, 0.0])
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh
}

pub fn build_mesh_im(
    mut empty: Mesh,
    data: &CompositeMeshData,
    stroke_tess: &mut StrokeTessellator,
    fill_tess: &mut FillTessellator,
) -> Mesh {
    for submeshdata in &data.mesh_data {
        let mut buffers = VertexBuffers::new();
        if let Some(path) = &submeshdata.tess_data.path {
            if let Some(options) = submeshdata.tess_data.fill {
                fill(&mut *fill_tess, &path, &options, &mut buffers);
            }
            if let Some(options) = submeshdata.tess_data.stroke {
                stroke(&mut *stroke_tess, &path, &options, &mut buffers);
            }
        }
        empty.merge(build_mesh(&buffers).with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![submeshdata.color.rgba_to_vec4(); buffers.vertices.len()],
        ));
    }
    empty
}
