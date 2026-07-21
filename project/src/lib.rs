use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub display: Display,
    pub resource: Resource,
    pub nodes: Vec<Node>,
    // pub scenes: Vec<Scene>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Display {
    pub width: u64,
    pub height: u64,
    pub display_mode: DisplayMode,
    pub fps_limit: u32,
    pub vsync: Vsync,
    pub rendering_api: RenderingAPI,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Vsync {
    Off,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DisplayMode {
    Windowed,
    Fullscreen,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RenderingAPI {
    WebGPU,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Resource {
    pub meshes: Vec<PathBuf>,
    pub textures: Vec<PathBuf>,
    pub shaders: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub name: String,
    pub id: u64,
    pub transform: [f32; 16],
    pub node_type: Type,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Type {
    Camera {
        fov: f32,
        skybox_texture_id: u64,
    },
    Entity {
        mesh_id: u64,
        texture_id: u64,
        shader_id: u64,
    },
}

// TODO later, when we have a bit more clue on
// how these need to be updated and how they
// can be useful.
// pub struct Scene {
//     pub name: String,
//     pub tree: Vec<Vec<TreeNode>>,
// }

// pub struct TreeNode {
//     pub node_id: u64,
//     pub children: Vec<u64>,
// }
