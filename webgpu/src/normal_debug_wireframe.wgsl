
struct Globals {
    view_m: mat4x4f,
    view_projection_m: mat4x4f,
    view_world_position: vec3f,
};

struct Instance {
    model_m: mat4x4f,
    normal_m: mat3x3f,
}

struct VertexData {
    position: vec4f,
    normal: vec3f,
    uv: vec2f,
}

@group(0)
@binding(0)
var<uniform> global: Globals;

@group(0)
@binding(1)
var<storage, read> instances: array<Instance>;

@group(0)
@binding(2)
var<storage, read> vertex_data: array<VertexData>;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
};

struct VSOutput {
    // The pixel position on the screen.
    @builtin(position) position: vec4f,
    // Will be interpolated and have to renormalized.
    @location(0) normal: vec3f,
    @location(1) @interpolate(flat) light_direction: vec3f,
    @location(2) @interpolate(linear, center) altitude: vec3f,
};

const world_light_direction = normalize(vec3(1.0, -1.0, -1.0));
// hard coding it for now
const WIDTH = f32(1920);
const HEIGHT = f32(1080);

@vertex
fn vs_main(vertex_input: VertexInput) -> VSOutput {
    var vsOut: VSOutput;

    let face_start_index = (vertex_input.vertex_index / 3) * 3;
    var vertices: array<vec4<f32>, 3>;

    var entity = instances[vertex_input.instance_index];
    // transform all vertices into clip space
    vertices[0] = global.view_projection_m * entity.model_m * vertex_data[face_start_index].position;
    vertices[1] = global.view_projection_m * entity.model_m * vertex_data[face_start_index + 1].position;
    vertices[2] = global.view_projection_m * entity.model_m * vertex_data[face_start_index + 2].position;
    // transform all vertices into NDC
    vertices[0] = vertices[0] / vertices[0].w;
    vertices[1] = vertices[1] / vertices[1].w;
    vertices[2] = vertices[2] / vertices[2].w;
    // transform all vertices into screen space
    var screen_space_vertices: array<vec2<f32>, 3>;
    screen_space_vertices[0] = (vertices[0].xy + vec2f(1.0)) * vec2f(0.5 * WIDTH, -0.5 * HEIGHT);
    screen_space_vertices[1] = (vertices[1].xy + vec2f(1.0)) * vec2f(0.5 * WIDTH, -0.5 * HEIGHT);
    screen_space_vertices[2] = (vertices[2].xy + vec2f(1.0)) * vec2f(0.5 * WIDTH, -0.5 * HEIGHT);
    let a = screen_space_vertices[2] - screen_space_vertices[1];
    let b = screen_space_vertices[2] - screen_space_vertices[0];
    let c = screen_space_vertices[1] - screen_space_vertices[0];
    // calculate triangle area using https://en.wikipedia.org/wiki/Exterior_algebra
    // we omit the division by two, because we would have to multiply by 2 when
    // calculating the altitudes
    let face_area: f32 = abs(b.x*c.y - b.y*c.x);
    // calculate screen space altitude for the triangle sides
    var screen_space_altitudes: array<vec3<f32>, 3>;
    screen_space_altitudes[0] = vec3<f32>(face_area / length(a), 0.0, 0.0);
    screen_space_altitudes[1] = vec3<f32>(0.0, face_area / length(b), 0.0);
    screen_space_altitudes[2] = vec3<f32>(0.0, 0.0, face_area / length(c));

    let vertex = vertex_data[vertex_input.vertex_index];
    // Compute the vertex position in device coordinates
    vsOut.position = global.view_projection_m * entity.model_m * vertex.position;
    // Orient the normals in world space
    vsOut.normal = entity.normal_m * vertex.normal;
    vsOut.light_direction = normalize((global.view_m * vec4f(world_light_direction, 0.0)).xyz);
    vsOut.altitude = screen_space_altitudes[vertex_input.vertex_index % 3];
    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    // All inter-stage variables get interpolated, so they
    // have to be renormalized if necessary.
    let normal = normalize(vsOut.normal);
    let light_direction = vsOut.light_direction;

    let similarity = (dot(-light_direction, normal) + 1.0) / 2.0;
    let albedo = mix(vec3f(0.0, 1.0, 0.0), vec3f(0.0, 0.0, 1.0), similarity);

    let distance_to_edge_px = min(vsOut.altitude.x, min(vsOut.altitude.y, vsOut.altitude.z));

    return select(vec4(0.0, 0.0, 0.0, 1.0), vec4(albedo, 1.0), distance_to_edge_px < 1.0);
}
