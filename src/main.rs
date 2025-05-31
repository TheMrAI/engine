use inner_app::InnerApp;
use winit::event_loop::{ControlFlow, EventLoop};

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
};

mod gpu;
mod inner_app;

#[derive(Default)]
struct App {
    app: Option<InnerApp>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // The Window should be created in this call, because the winit documentation states that this
        // is the only point which they could guarantee proper initialization on all supported platforms.
        self.app = Some(InnerApp::new(event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId, // we only have one window
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.
                if let Some(app) = self.app.as_mut() {
                    app.gpu.render();
                    app.window.request_redraw();
                }
                // else nothing to do yet
            }
            WindowEvent::Focused(_) => {}
            WindowEvent::CursorEntered { device_id: _ } => {}
            WindowEvent::CursorLeft { device_id: _ } => {}
            WindowEvent::Resized(inner_resolution) => {
                // Recreate the surface texture according to the new inner physical resolution.
                if let Some(app) = self.app.as_mut() {
                    let config = app
                        .gpu
                        .surface
                        .get_default_config(
                            &app.gpu.adapter,
                            inner_resolution.height,
                            inner_resolution.width,
                        )
                        .unwrap();
                    app.gpu.surface.configure(&app.gpu.device, &config);
                }
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        _event: DeviceEvent,
    ) {
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    // event_loop.set_control_flow(ControlFlow::Poll);
    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
