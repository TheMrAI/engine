
struct Uniforms {
    normalMatrix: mat3x3f,
    worldViewProjection: mat4x4f,
    // normal_matrix: mat3x3f,
    light_color: vec4f,
    light_direction: vec3f,
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
    // @location(1) color: vec4f,
};

@group(0)
@binding(0)
var<uniform> uni: Uniforms;

@vertex
fn vs_main(vertex: Vertex) -> VSOutput {
    var vsOut: VSOutput;

    vsOut.position = uni.worldViewProjection * vertex.position;
    vsOut.normal = uni.normalMatrix * vertex.normal;
    // vsOut.color = vertex.color;
    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    let normal = normalize(vsOut.normal);
    let light = dot(normal, -uni.light_direction);

    let color = uni.light_color.rgb * light;
    return vec4f(color, uni.light_color.a);
}