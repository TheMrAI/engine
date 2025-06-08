
struct Uniforms {
    color: vec4f,
    matrix: mat4x4f,
};

struct Vertex {
    @location(0) position: vec4f,
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

    vsOut.position = uni.matrix * vertex.position;

    return vsOut;
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4<f32> {
    return uni.color;
}