
struct Uniforms {
    color: vec4f,
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

    var transformed = uni.matrix * vec3f(vertex.position,1.0);
    vsOut.position = vec4f(transformed.xy, 0.0, 1.0);

    return vsOut;
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4<f32> {
    return uni.color;
}