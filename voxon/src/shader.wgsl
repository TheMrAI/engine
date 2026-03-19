
struct Globals {
    view_projection: mat4x4f,
    view_world_position: vec3f,
};

struct Entity {
    world: mat4x4f,
    normal: mat3x3f,
}

@group(0)
@binding(0)
var<uniform> global: Globals;

@group(1)
@binding(0)
var<uniform> entity: Entity;

struct Vertex {
    // The position of the vertex.
    @location(0) position: vec4f,
    @location(1) normal: vec3f,
};

struct VSOutput {
    // The pixel position on the screen.
    @builtin(position) position: vec4f,
    // Will be interpolated and have to renormalized.
    @location(0) normal: vec3f,
};

@vertex
fn vs_main(vertex: Vertex) -> VSOutput {
    var vsOut: VSOutput;

    // Compute the vertex position in device coordinates
    vsOut.position = global.view_projection * entity.world * vertex.position;

    // Orient the normals in world space
    vsOut.normal = entity.normal * vertex.normal;

    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    // All inter-stage variables get interpolated, so they
    // have to be renormalized if necessary.
    let normal = normalize(vsOut.normal);

    return vec4f(0.2, 0.2, 0.2, 1.0);
}
