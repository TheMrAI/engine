
struct Uniforms {
    // This inverse must not contain the transition
    // from view matrix, only the rotation and scaling. The projection
    // should be left untouched.
    inverse_view_projection: mat4x4f,
}

struct VertexOutput {
    // The position is a mandatory field.
    @builtin(position) position: vec4<f32>,
    // Normalized device space position
    @location(0) ndc_position: vec4<f32>,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var texture_sampler: sampler;

@group(0)
@binding(2)
var texture: texture_cube<f32>;

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

    vertex_output.position = vec4f(f32(clip_x), f32(clip_y), 1.0, 1.0);
    // Position is simply copied, because we need to modify it in fragment
    // shader, but builtin position is automatically transformed before reaching
    // the fragment shader.
    vertex_output.ndc_position = vertex_output.position;

    return vertex_output;
}

@fragment
fn fs_main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    // Not entirely true. This won't be the views position in the world as we have discarded it's
    // transposition, but it will be something like that.
    var world_look_position = uniforms.inverse_view_projection * vertex_output.ndc_position;
    // This will be the look direction in world space though.
    var world_look_direction = normalize(world_look_position.xyz / world_look_position.w) * vec3f(1.0, 1.0, -1.0);

    return textureSample(texture, texture_sampler, world_look_direction);
}
