mod cube_map;
mod normal_debug;
mod normal_debug_wireframe;
mod textured_draw;

use std::{cell::RefCell, rc::Rc};

pub use cube_map::CubeMap;
use lina::matrix::Matrix;
pub use normal_debug::NormalDebug;
pub use normal_debug_wireframe::NormalDebugWireframe;
use scene::MeshNode;
pub use textured_draw::TexturedDraw;

use crate::MeshBuffer;

pub trait Pipeline {
    fn schedule_render(&mut self, mesh_node: Rc<RefCell<MeshNode>>);

    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        mesh_cache: &std::collections::HashMap<u32, MeshBuffer>,
        texture_cache: &std::collections::HashMap<u32, wgpu::Texture>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_matrix: &Matrix<f32, 4, 4>,
        _view_projection_matrix: &Matrix<f32, 4, 4>,
        global_uniform_buffer: &wgpu::Buffer,
    );
}
