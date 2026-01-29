
struct Globals {
    view_projection: mat4x4f,
    light_color: vec4f,
    light_position: vec3f,
    view_world_position: vec3f,
    shininess: f32,
    light_direction: vec3f,
    limit: f32,
};

struct Entity {
    world: mat4x4f,
    normal: mat3x3f,
}

@group(0)
@binding(0)
var<uniform> global: Globals;

@group(1)
@binding(0)
var<uniform> entity: Entity;

struct Vertex {
    // The position of the vertex.
    @location(0) position: vec4f,
    @location(1) normal: vec3f,
};

struct VSOutput {
    // The pixel position on the screen.
    @builtin(position) position: vec4f,
    // Will be interpolated and have to renormalized.
    @location(0) normal: vec3f,
    @location(1) surface_to_light: vec3f,
    @location(2) surface_to_view: vec3f,
};

@vertex
fn vs_main(vertex: Vertex) -> VSOutput {
    var vsOut: VSOutput;

    // Compute the vertex position in device coordinates
    vsOut.position = global.view_projection * entity.world * vertex.position;
    
    // Orient the normals in world space
    vsOut.normal = entity.normal * vertex.normal;
    
    // Compute surface_to_light vector in world space
    let surface_world_position = (entity.world * vertex.position).xyz;
    vsOut.surface_to_light = global.light_position - surface_world_position;

    // Compute the surface_to_view vector in world space
    vsOut.surface_to_view = global.view_world_position - surface_world_position;

    // the returned vector will automatically be normalized using w
    // [x,y,z,w] => [x/w, y/w, z/w, 1]
    return vsOut;
}

@fragment
fn fs_main(vsOut: VSOutput) -> @location(0) vec4<f32> {
    // All inter-stage variables get interpolated, so they
    // have to be renormalized if necessary.
    let normal = normalize(vsOut.normal);

    let surface_to_light_direction = normalize(vsOut.surface_to_light);
    
    let surface_to_view_direction = normalize(vsOut.surface_to_view);
    let half_vector = normalize(surface_to_light_direction + surface_to_view_direction);
    
    var light = 0.0;
    var specular = 0.0;

    let dot_from_direction = dot(surface_to_light_direction, -global.light_direction);
    if (dot_from_direction > global.limit) {
        light = dot(normal, surface_to_light_direction);

        specular = dot(normal, half_vector);   
        specular = select(0.0, pow(specular, global.shininess), specular > 0.0);
    }

    let color = global.light_color.rgb * light + specular;
    return vec4f(color, global.light_color.a);
}