use std::{fs, sync::Arc};

use project::Root;
use webgpu::RenderServer;
use winit::window::Window;

use crate::game::Game;

pub(super) struct InnerApp {
    pub window: Arc<Window>,
    pub game: Game,
}

impl InnerApp {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let data =
            fs::read_to_string("/home/mrai/Documents/tinker/engine/voxon/resources/project.toml")
                .unwrap();
        let project = toml::from_str::<Root>(&data).unwrap();

        let window_attributes = Window::default_attributes()
            .with_title("Voxon")
            .with_resizable(false)
            .with_inner_size(winit::dpi::LogicalSize::new(
                project.display.width as f32,
                project.display.height as f32,
            ));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let webgpu = pollster::block_on(RenderServer::new(Arc::clone(&window)));
        let game = Game::new(webgpu);

        InnerApp { window, game }
    }
}
