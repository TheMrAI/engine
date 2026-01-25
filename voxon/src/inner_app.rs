use std::{sync::Arc, time::Duration};

use graphic::camera::Camera;
use winit::window::Window;

use crate::gpu::Wgpu;

pub(super) struct InnerApp {
    pub window: Arc<Window>,
    pub gpu: Wgpu,
    pub camera: Camera,
    pub prev_render_time: std::time::Instant,
    // A dirty hack for managing the Cube's rotation
    // state.
    pub delta_t_for_cube: std::time::Duration,
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
            prev_render_time: std::time::Instant::now(),
            delta_t_for_cube: Duration::default(),
        }
    }
}
