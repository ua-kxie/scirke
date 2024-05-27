use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, PolygonMode, ShaderRef},
    sprite::Material2d,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SchematicMaterial {
    #[uniform(0)]
    pub color: Color,
}

impl Material2d for SchematicMaterial {
    fn vertex_shader() -> ShaderRef {
        "schematic_shader.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "schematic_shader.wgsl".into()
    }
    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
