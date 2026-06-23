
struct Globals {
    view_projection_m: mat4x4f,
    view_world_position: vec3f,
};

struct Instance {
    model_m: mat4x4f,
    normal_m: mat3x3f,
}

struct TextureInstance {
    texture_scale: f32,
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
var<storage, read> texture_instances: array<TextureInstance>;

@group(0)
@binding(4)
var texture_sampler: sampler;

@group(0)
@binding(5)
var texture: texture_2d<f32>;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
};

struct VSOutput {
    // The pixel position on the screen.
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_main(vertex_input: VertexInput) -> VSOutput {
    var vsOut: VSOutput;

    var instance = instances[vertex_input.instance_index];
    var vertex = vertex_data[vertex_input.vertex_index];
    // Compute the vertex position in device coordinates
    vsOut.position = global.view_projection_m * instance.model_m * vertex.position;

    // Pass uv.
    var texture_instances = texture_instances[vertex_input.instance_index];
    vsOut.uv = vertex.uv * texture_instances.texture_scale;

    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, vsOut.uv);
}
