
struct Uniforms {
    world: mat4x4f,
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
    vsOut.normal = (uni.world * vec4f(vertex.normal, 0)).xyz;;
    // vsOut.normal = uni.normal_matrix * vertex.normal;
    // vsOut.normal = (uni.matrix * vec4f(vertex.normal, 0.0)).xyz;
    // vsOut.color = vertex.color;
    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

// @fragment
// fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
//     // let light_direction = normalize(vec3f(-1, -1.0, -1.0));
//     let light_direction = normalize(uni.matrix * vec4f(-1, -1.0, -1.0, 0)).xyz;

//     let normal = normalize(vsOut.normal);
//     let light_intensity = dot(normal, -light_direction);

//     return vec4f(vec3f(1.0, 1.0, 1.0) * light_intensity, 1.0);
//     // let color = vsOut.color.rgb * light_intensity;
//     // return vec4f(color, vsOut.color.a);
// }

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    let normal = normalize(vsOut.normal);
    let light = dot(normal, -uni.light_direction);

    let color = uni.light_color.rgb * light;
    return vec4f(color, uni.light_color.a);
}