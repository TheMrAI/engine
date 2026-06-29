struct Uniforms {
    view_projection: mat4x4f,
}

struct Vertex {
    @location(0) position: vec4f,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) normal: vec3f,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var texture_sampler: sampler;

@group(0)
@binding(2)
var texture: texture_cube<f32>;

@vertex
fn vs_main(vertex: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;

    // To keep the code a bit simpler we cheated here and relied on the fact, that
    // the default cube mesh is generated at the origo, so all its vertex positions
    // can be directly used as normal vectors (after normalizing only the xyz of course).
    // These normal vectors are special in a sense, that they will describe a sphere after
    // interpolation. Exactly what is necessary for cubemapping.
    vertex_output.position = uniforms.view_projection * vertex.position;
    vertex_output.normal = normalize(vertex.position.xyz);

    return vertex_output;
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, normalize(vertex_output.normal));
}
