use std::sync::Arc;

use graphic::camera::Camera;
use winit::window::Window;

use crate::gpu::Wgpu;

pub(super) struct InnerApp {
    pub window: Arc<Window>,
    pub gpu: Wgpu,
    pub camera: Camera,
}

impl InnerApp {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let window_attributes = Window::default_attributes()
            .with_title("Strategos")
            .with_resizable(false)
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let gpu = pollster::block_on(Wgpu::new(Arc::clone(&window)));

        let camera = Camera::default();

        InnerApp {
            window,
            gpu,
            camera,
        }
    }
}
