use std::path::PathBuf;

use project::{Display, Node, RenderingAPI::WebGPU, Resource, Root, Type};

fn main() {
    let display = Display {
        width: 1024,
        height: 768,
        display_mode: project::DisplayMode::Fullscreen,
        fps_limit: 120,
        vsync: project::Vsync::Off,
        rendering_api: WebGPU,
    };

    let resource = Resource {
        meshes: vec![PathBuf::from(
            "/home/mrai/Documents/tinker/engine/voxon/resources/meshes/stanford_dragon_flat_17k.obj",
        )],
        textures: vec![PathBuf::from(
            "/home/mrai/Documents/tinker/engine/voxon/resources/textures/cube_atlas.png",
        )],
        shaders: vec![PathBuf::from(
            "/home/mrai/Documents/tinker/engine/voxon/src/mipmap.wgsl",
        )],
    };

    let nodes = vec![
        Node {
            name: String::from("Camera"),
            id: 1,
            transform: [0.0; 16],
            node_type: Type::Camera {
                fov: 70.0,
                skybox_texture_id: 0,
            },
        },
        Node {
            name: String::from("Plane"),
            id: 2,
            transform: [0.0; 16],
            node_type: Type::Entity {
                mesh_id: 1,
                texture_id: 2,
                shader_id: 3,
            },
        },
    ];

    let project_root = Root {
        display,
        resource,
        nodes,
    };

    let as_toml_shit = toml::to_string(&project_root).unwrap();
    println!("From example TOML with love: \n{}", as_toml_shit);
}
