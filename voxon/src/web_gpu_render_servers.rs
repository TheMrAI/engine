use std::sync::Arc;

use graphic::camera::Camera;
use wgpu::{Device, ExperimentalFeatures, Queue, Surface};
use winit::{dpi::PhysicalSize, window::Window};

use crate::scene::Scene;

#[derive(Debug)]
pub struct WebGpuRenderServer {
    // TODO remove this as well,
    // only relevant for the camera which
    // should become a node later anyways
    inner_size: PhysicalSize<u32>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    global_uniform_buffer: wgpu::Buffer,
    // TODO detach this
    scene: Scene,
}

impl WebGpuRenderServer {
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

        // Global Uniform buffer
        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            // uniforms have to be padded to a multiple of 8
            #[allow(clippy::identity_op)] // for clearer explanation
            size: (16 + 16 + 3 + 1) * 4, // (view matrix, view projection matrix, view_world_position, pad) * float size
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scene = Scene::new(&adapter, &surface, &device, &queue, &global_uniform_buffer);

        WebGpuRenderServer {
            inner_size,
            surface,
            device,
            queue,
            global_uniform_buffer,
            scene,
        }
    }

    pub fn render(&mut self, camera: &Camera, wireframe: bool) {
        // the camera matrix
        let look_at = camera.as_transform_matrix();
        // view matrix
        let view_matrix = look_at;

        let aspect_ratio = self.inner_size.width as f32 / self.inner_size.height as f32;
        let projection_matrix = graphic::transform::perspective_proj_sym_h_fov(
            // PI / 2.0, 90 deg FOV
            (std::f32::consts::PI / 180.0) * 75.0, // 75 degree FOV
            aspect_ratio,
            -0.05,
            -4000.0,
        );

        // Render the rest
        let view_projection_matrix = projection_matrix * view_matrix;

        // It does not matter if it is rendered first or last, because
        // the skybox is at Z value 1.0 in NDC.
        // No other draw call, should write if the depth value equals 1.0.
        let translation_free_view_matrix = {
            let mut tmp = view_matrix;
            tmp[(0, 3)] = 0.0;
            tmp[(1, 3)] = 0.0;
            tmp[(2, 3)] = 0.0;
            tmp
        };
        let translation_free_view_projection_matrix =
            projection_matrix * translation_free_view_matrix;

        let transposed_view_matrix = view_matrix.transpose();
        let transposed_view_projection_matrix = view_projection_matrix.transpose();

        // Update Uniforms
        let global_uniforms = transposed_view_matrix
            .as_slices()
            .iter()
            .flatten()
            .flat_map(|entry| entry.to_le_bytes())
            .chain(
                transposed_view_projection_matrix
                    .as_slices()
                    .iter()
                    .flatten()
                    .flat_map(|entry| entry.to_le_bytes()),
            )
            .chain(
                [camera.eye()[0], camera.eye()[1], camera.eye()[2], 0.0]
                    .iter()
                    .flat_map(|entry| entry.to_le_bytes()),
            )
            .collect::<Vec<u8>>();

        self.queue
            .write_buffer(&self.global_uniform_buffer, 0, &global_uniforms);

        // Create render texture
        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swap-chain texture");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: frame.texture.size(),
            mip_level_count: 1, // no extra mips, has to be 1
            sample_count: 1,    // no multisampling, so 1
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[], // no special view format needed
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.scene.render(
                &mut render_pass,
                &view_matrix,
                &view_projection_matrix,
                &translation_free_view_projection_matrix,
                &self.queue,
                camera,
                wireframe,
            );
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
