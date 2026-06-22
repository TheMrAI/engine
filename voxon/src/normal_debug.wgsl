
struct Globals {
    view_m: mat4x4f,
    view_projection_m: mat4x4f,
};

struct Instance {
    model_m: mat4x4f,
    normal_m: mat3x3f,
    // Here is where we could
    // put LOD offsets.
}

struct VertexData {
    position: vec4f,
    normal: vec3f,
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

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
};

struct VSOutput {
    // The pixel position on the screen.
    @builtin(position) position: vec4f,
    // Will be interpolated and have to be renormalized.
    @location(0) normal: vec3f,
    @location(1) @interpolate(flat) light_direction: vec3f,
};

const world_light_direction = normalize(vec3(1.0, -1.0, -1.0));

@vertex
fn vs_main(vertex_input: VertexInput) -> VSOutput {
    var vsOut: VSOutput;

    var instance = instances[vertex_input.instance_index];
    var vertex = vertex_data[vertex_input.vertex_index];
    // Compute the vertex position in Clip space
    vsOut.position = global.view_projection_m * instance.model_m * vertex.position;

    // Orient the normals in world space
    vsOut.normal = instance.normal_m * vertex.normal;
    vsOut.light_direction = normalize((global.view_m * vec4f(world_light_direction, 0.0)).xyz);
    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    // All inter-stage variables get interpolated, so they
    // have to be renormalized if necessary.
    let normal = normalize(vsOut.normal);
    let light_direction = vsOut.light_direction;

    let similarity = (dot(-light_direction, normal) + 1.0) / 2.0;
    let albedo = mix(vec3f(0.0, 1.0, 0.0), vec3f(0.0, 0.0, 1.0), similarity);

    return vec4(albedo, 1.0);
}
