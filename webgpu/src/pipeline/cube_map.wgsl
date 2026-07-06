
struct Globals {
    view_m: mat4x4f,
    view_projection_m: mat4x4f,
    view_world_position: vec3f,
};

struct Instance {
    model_m: mat4x4f,
    normal_m: mat3x3f,
}

struct VertexData {
    position: vec4f,
    normal: vec3f,
    uv: vec2f,
}

@group(0)
@binding(0)
var<uniform> global: Globals;

@group(0)
@binding(1)
var<storage, read> instances: array<Instance>;

@group(0)
@binding(2)
var<storage, read> vertex_data: array<VertexData>;

@group(0)
@binding(3)
var texture_sampler: sampler;

@group(0)
@binding(4)
var texture: texture_cube<f32>;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) sampling_normal: vec3f,
}

@vertex
fn vs_main(vertex_input: VertexInput) -> VertexOutput {
    var vsOut: VertexOutput;

    var instance = instances[vertex_input.instance_index];
    var vertex = vertex_data[vertex_input.vertex_index];
    // Compute the vertex position in Clip space
    vsOut.position = global.view_projection_m * instance.model_m * vertex.position;

    // For a cube map, the we aren't interested in world space normals, but
    // the regular normals of the smooth shaded cube.
    // It makes no difference how the cube is translated/rotated.
    vsOut.sampling_normal = vertex.normal;

    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, normalize(vertex_output.sampling_normal));
}
