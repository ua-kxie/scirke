#import bevy_sprite::mesh2d_functions::get_model_matrix

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> transform: mat4x4<f32>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // out.clip_position = transform * vec4(vertex.position, 1.0);
    out.clip_position = get_model_matrix(vertex.instance_index) * vec4(vertex.position, 1.0);
    return out;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return color;
}