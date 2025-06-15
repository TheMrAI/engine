
struct Uniforms {
    matrix: mat4x4f,
};

struct Vertex {
    @location(0) position: vec4f,
    @location(1) color: vec4f,
};

struct VSOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
};

@group(0)
@binding(0)
var<uniform> uni: Uniforms;

@vertex
fn vs_main(vertex: Vertex) -> VSOutput {
    var vsOut: VSOutput;

    vsOut.position = uni.matrix * vertex.position;
    vsOut.color = vertex.color;
    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    return vsOut.color;
}