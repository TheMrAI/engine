
struct Uniforms {
    normalMatrix: mat3x3f,
    worldViewProjection: mat4x4f,
    world: mat4x4f,
    light_color: vec4f,
    light_position: vec3f,
};

struct Vertex {
    // The position of the vertex.
    @location(0) position: vec4f,
    @location(1) normal: vec3f,
    // @location(2) color: vec4f,
};

struct VSOutput {
    // The pixel position on the screen.
    @builtin(position) position: vec4f,
    // Will be interpolated and have to renormalized.
    @location(0) normal: vec3f,
    @location(1) surface_to_light: vec3f,
};

@group(0)
@binding(0)
var<uniform> uni: Uniforms;

@vertex
fn vs_main(vertex: Vertex) -> VSOutput {
    var vsOut: VSOutput;

    // Compute the vertex position in device coordinates
    vsOut.position = uni.worldViewProjection * vertex.position;
    // Orient the normals in world space
    vsOut.normal = uni.normalMatrix * vertex.normal;
    // Compute surface_to_light vector in world space
    let surface_world_position = (uni.world * vertex.position).xyz;
    vsOut.surface_to_light = uni.light_position - surface_world_position;

    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    // All inter-stage variables get interpolated, so they
    // have to be renormalized if necessary.
    let normal = normalize(vsOut.normal);

    let surface_to_light_direction = normalize(vsOut.surface_to_light);
    let light = clamp(dot(normal, surface_to_light_direction), 0, 1);

    let color = uni.light_color.rgb * light;
    return vec4f(color, uni.light_color.a);
}