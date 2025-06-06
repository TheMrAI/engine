
struct Uniforms {
    color: vec4f,
    resolution: vec2f,
    matrix: mat3x3f,
};

struct Vertex {
    @location(0) position: vec2f,
};

struct VSOutput {
    @builtin(position) position: vec4f,
};

@group(0)
@binding(0)
var<uniform> uni: Uniforms;

@vertex
fn vs_main(vertex: Vertex) -> VSOutput {
    var vsOut: VSOutput;

    // matrix transformation
    let position = (uni.matrix * vec3f(vertex.position, 1)).xy;

    // transform the position into clip space
    let zero_to_one = position / uni.resolution;
    let scale = zero_to_one * 2;
    let shifted = scale - 1.0;
    let clip_space = shifted * vec2f(1, -1);

    vsOut.position = vec4f(clip_space, 0.0, 1.0);

    return vsOut;
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4<f32> {
    return uni.color;
}