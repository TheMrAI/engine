
// We can do this to have the four vertices or make a buffer, fill it etc..
// This is simpler.
const vertices = array<vec4f, 3>(vec4f(-0.5, -0.5, 0.0, 1.0), vec4f(0.0, 0.5, 0.0, 1.0), vec4f(0.5, -0.5, 0.0, 1.0));

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4f {
    var position: vec4f;
    // This is a weird trick, necessary as Naga doesn't allow indexing of arrays
    // with none constant values.
    if in_vertex_index == 0 {
        position = vertices[0];
    } else if in_vertex_index == 1 {
        position = vertices[1];
    } else if in_vertex_index == 2 {
        position = vertices[2];
    }

    return position;
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4<f32> {
    return vec4f(0.0, 1.0, 0.0, 1.0);
}