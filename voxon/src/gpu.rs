use std::sync::Arc;

use graphic::camera::Camera;
use wgpu::{Adapter, Device, ExperimentalFeatures, Queue, Surface};
use winit::{dpi::PhysicalSize, window::Window};

use crate::scene::Scene;

pub struct Wgpu {
    pub inner_size: PhysicalSize<u32>,
    pub adapter: Adapter,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub scene: Scene,
    frametimes: frametime::Sampler<1024>,
    elapsed_time: std::time::Duration,
}

impl Wgpu {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let inner_size = window.inner_size();
        let surface = instance.create_surface(window).unwrap();
        // Request an adapter that can support our surface
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create logical device and command queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("gpu_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: ExperimentalFeatures::disabled(),
            })
            .await
            .expect("Failed to create device");
        println!("Prepared device: {device:?}",);

        // Configure surface
        let config = surface
            .get_default_config(&adapter, inner_size.width, inner_size.height)
            .unwrap();
        surface.configure(&device, &config);

        let scene = Scene::new(&adapter, &surface, &device, &queue);

        Wgpu {
            inner_size,
            adapter,
            surface,
            device,
            queue,
            scene,
            frametimes: frametime::Sampler::new(),
            elapsed_time: std::time::Duration::default(),
        }
    }

    pub fn render(&mut self, camera: &Camera, delta_t: std::time::Duration) {
        self.frametimes.add_frametime(delta_t.as_nanos());
        self.elapsed_time += delta_t;

        if self.elapsed_time > std::time::Duration::from_secs(1) {
            self.elapsed_time -= std::time::Duration::from_secs(1);
            let stats = self.frametimes.stats();
            println!("{}", stats);
        }

        self.scene.simulate(delta_t);
        self.scene.render(
            &self.inner_size,
            &self.surface,
            &self.device,
            &self.queue,
            camera,
        );
    }
}
