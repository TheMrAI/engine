use inner_app::InnerApp;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::{ControlFlow, EventLoop};

use winit::keyboard::PhysicalKey;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
};

mod cube_map;
mod game;
mod inner_app;
mod mesh;
mod normal_debug;
mod normal_debug_instanced;
mod normal_debug_wireframe;
mod scene;
mod skybox;
mod texture;
mod textured_draw;
mod webgpu;

#[derive(Default)]
struct App {
    app: Option<InnerApp>,
    focused: bool,
    navigating: bool,
    // stores for each key if it is currently being pressed/held or not
    key_state: std::collections::BTreeMap<winit::keyboard::KeyCode, bool>,
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

                // Cycle game loop.
                // TODO: This is not an optimal setup. We should be the ones in control of the main
                // thread.
                if let Some(app) = self.app.as_mut() {
                    // app.gpu.render(&app.camera, delta_t, self.wireframe);
                    app.game.run_loop(&self.key_state);
                    // for continuous rendering
                    app.window.request_redraw();
                }
                // else nothing to do yet
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    // If focus is lost from the application
                    // we simply clear all keys. Resetting the state.
                    // Otherwise the user could click away while
                    // navigating, then release all key, and keep moving in the
                    // last read direction.
                    self.key_state.clear();
                }
                self.focused = focused
            }
            WindowEvent::CursorEntered { device_id: _ } => {}
            WindowEvent::CursorLeft { device_id: _ } => {}
            WindowEvent::Resized(_) => {
                // TODO if/when necessary
                // Recreate the surface texture according to the new inner physical resolution.
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                // camera navigation controls for the engine
                if self.focused
                    && self.navigating
                    && let PhysicalKey::Code(key_code) = event.physical_key
                {
                    let is_pressed = event.state == ElementState::Pressed;
                    self.key_state
                        .entry(key_code)
                        .and_modify(|entry| *entry = is_pressed)
                        .or_insert(is_pressed);
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                if self.focused && matches!(button, MouseButton::Right) {
                    match state {
                        ElementState::Pressed => self.navigating = true,
                        ElementState::Released => {
                            self.navigating = false;
                            // If 'navigation' is stopped
                            // we simply clear all keys. Resetting the state.
                            // Otherwise the user could release the 'navigation' key while
                            // navigating, then release all key, and keep moving in the
                            // last read direction.
                            self.key_state.clear();
                        }
                    }
                }
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _, // touchpad ignored
            } => {
                if self.focused && self.navigating {
                    match delta {
                        MouseScrollDelta::LineDelta(_dx, dy) => {
                            if let Some(app) = self.app.as_mut() {
                                app.game.mouse_wheel(dy);
                            }
                        }
                        MouseScrollDelta::PixelDelta(_) => {}
                    }
                }
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        #[allow(clippy::single_match)]
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.focused
                    && self.navigating
                    && let Some(app) = self.app.as_mut()
                {
                    app.game.mouse_motion(delta);
                }
            }
            _ => (), // the rest we don't care
        }
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
