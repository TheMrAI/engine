
struct VertexOutput {
    // The position is a mandatory field.
    @builtin(position) position: vec4<f32>,
    @location(0) uv_coord: vec2<f32>,
}

@group(0)
@binding(0)
var texture_sampler: sampler;

@group(0)
@binding(1)
var texture: texture_2d<f32>;

// Expected to be called with vertex indexes: 0, 1, 2
//
// Based on the index, it generates a triangle in clip space such
// that it covers the whole space.
// Triangle marked by *-s, clipspace is the rectangle within.
//
//  * <- C (-1, 3), UV(0, -1)
//    *
//      *
//        *
//          *
//            *
// ------------ * (1, 1), UV(1, 0)
// |          |   *
// |          |     *
// |          |       *
// |          |         *
// ------------ * * * * * * <- B (3, -1), UV(2, 1)
// ^
// A (-1, -1), UV(0, 1)
//
// This way the midpoint between B and C is at (1, 1)
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var vertex_output: VertexOutput;

    // Using the bitmask of numbers 0, 1, 2 we identify which point we want to generate.
    // A: 0 (00)
    // B: 1 (01)
    // C: 2 (10)
    var clip_x = -1 + (4 * (1 & i32(vertex_index)));
    var clip_y = -1 + (4 * i32(vertex_index / 2));

    var u = (clip_x + 1) / 2;
    var v = 1 - (2 * (i32(vertex_index) / 2));

    vertex_output.position = vec4f(f32(clip_x), f32(clip_y), 0.0, 1.0);
    vertex_output.uv_coord = vec2f(f32(u), f32(v));

    return vertex_output;
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, vertex_output.uv_coord);
}
