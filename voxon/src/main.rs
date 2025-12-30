use inner_app::InnerApp;
use winit::event::{ElementState, MouseScrollDelta};
use winit::event_loop::{ControlFlow, EventLoop};

use winit::keyboard::PhysicalKey;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
};

mod gpu;
mod inner_app;

struct App {
    app: Option<InnerApp>,
    focused: bool,
    navigating: bool,
    speed: f32, // speed in m/s
    // stores for each key if it is currently being pressed/held or not
    key_state: std::collections::BTreeMap<winit::keyboard::KeyCode, bool>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            app: None,
            focused: false,
            navigating: false,
            speed: 1.0,
            key_state: Default::default(),
        }
    }
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

                // Before redraw, apply all navigation changes.
                let key_left_shift = self
                    .key_state
                    .get(&winit::keyboard::KeyCode::ShiftLeft)
                    .cloned()
                    .unwrap_or(false);
                let key_right_shift = self
                    .key_state
                    .get(&winit::keyboard::KeyCode::ShiftRight)
                    .cloned()
                    .unwrap_or(false);
                let key_w = self
                    .key_state
                    .get(&winit::keyboard::KeyCode::KeyW)
                    .cloned()
                    .unwrap_or(false);
                let key_s = self
                    .key_state
                    .get(&winit::keyboard::KeyCode::KeyS)
                    .cloned()
                    .unwrap_or(false);
                let key_d = self
                    .key_state
                    .get(&winit::keyboard::KeyCode::KeyD)
                    .cloned()
                    .unwrap_or(false);
                let key_a = self
                    .key_state
                    .get(&winit::keyboard::KeyCode::KeyA)
                    .cloned()
                    .unwrap_or(false);
                let key_e = self
                    .key_state
                    .get(&winit::keyboard::KeyCode::KeyE)
                    .cloned()
                    .unwrap_or(false);
                let key_q = self
                    .key_state
                    .get(&winit::keyboard::KeyCode::KeyQ)
                    .cloned()
                    .unwrap_or(false);

                // Draw.
                if let Some(app) = self.app.as_mut() {
                    let current_time = std::time::Instant::now();
                    let delta_t = current_time.duration_since(app.prev_render_time);

                    let elapsed_s = delta_t.as_secs_f32();
                    let speed = if key_left_shift || key_right_shift {
                        3.0 * self.speed * elapsed_s
                    } else {
                        1.0 * self.speed * elapsed_s
                    };

                    if key_w {
                        app.camera.move_on_look_at_vector(speed);
                    };
                    if key_s {
                        app.camera.move_on_look_at_vector(-speed);
                    };
                    if key_d {
                        app.camera.move_on_right_vector(speed);
                    };
                    if key_a {
                        app.camera.move_on_right_vector(-speed);
                    };
                    if key_e {
                        app.camera.move_on_up_vector(speed);
                    };
                    if key_q {
                        app.camera.move_on_up_vector(-speed);
                    }

                    app.gpu
                        .render(&app.camera, delta_t, &mut app.delta_t_for_cube);
                    // for continuos rendering
                    app.window.request_redraw();

                    // Dirty update time
                    app.prev_render_time = current_time;
                }
                // else nothing to do yet
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    // If focus is lost from the application
                    // we simply clear all keys. Resetting the state.
                    // Otherwise the user could click away while holding a
                    // key, let go of the key, but that will no longer be
                    // registered by the application.
                    self.key_state.clear();
                }
                self.focused = focused
            }
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
        event: DeviceEvent,
    ) {
        #[allow(clippy::single_match)]
        match event {
            DeviceEvent::Key(key_event) => {
                // camera navigation controls for the engine
                if self.focused && self.navigating {
                    match key_event.physical_key {
                        PhysicalKey::Code(key_code) => {
                            let is_pressed = key_event.state == ElementState::Pressed;
                            self.key_state
                                .entry(key_code)
                                .and_modify(|entry| *entry = is_pressed)
                                .or_insert(is_pressed);
                        }
                        _ => {}
                    }
                }
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.focused && self.navigating {
                    if let Some(app) = self.app.as_mut() {
                        // Negate all inputs, inverting the movements
                        app.camera.pitch(-delta.1 as f32 / 50.0);
                        app.camera.yaw(-delta.0 as f32 / 50.0);
                    }
                }
            }
            DeviceEvent::MouseWheel { delta } => {
                if self.focused && self.navigating {
                    match delta {
                        MouseScrollDelta::LineDelta(_dx, dy) => {
                            // To change the speed we use a logarithm function as
                            // those types of inputs fell much more natural.
                            // Shift it by 1 to the left so it reaches zero at zero,
                            // then flatten the result by half.
                            // This way within the range os 0.1 - 30 the user
                            // gets finer control on the lower ends and coarser on the
                            // higher ends.
                            self.speed += dy * ((self.speed + 1.0).log2() / 2.0);
                            self.speed = self.speed.clamp(0.1, 30.0);
                        }
                        MouseScrollDelta::PixelDelta(_) => {}
                    }
                }
            }
            DeviceEvent::Button { button, state } => {
                if self.focused && button == 1 {
                    match state {
                        ElementState::Pressed => self.navigating = true,
                        ElementState::Released => self.navigating = false,
                    }
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
