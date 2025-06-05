use std::{borrow::Cow, sync::Arc};

use wgpu::{
    Adapter, BindGroupEntry, BufferBinding, BufferUsages, Device, Queue, ShaderModule, Surface,
    VertexAttribute, VertexBufferLayout,
};
use winit::window::Window;

pub struct Wgpu {
    pub adapter: Adapter,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub shader: ShaderModule,
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
            })
            .await
            .expect("Failed to create device");
        println!("Prepared device: {:?}", device);

        // Configure surface
        let config = surface
            .get_default_config(&adapter, inner_size.width, inner_size.height)
            .unwrap();
        surface.configure(&device, &config);

        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        Wgpu {
            adapter,
            surface,
            device,
            queue,
            shader,
        }
    }

    pub fn render(&mut self, window: Arc<Window>) {
        // Uniform buffer
        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            // uniforms have to be padded to a multiple of 32
            size: (4 + 2 + 2) * 4_u64, // (color + resolution + translation) * float32 + padding
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let resolution = window.inner_size();
        let uniforms: Vec<f32> = vec![
            //color: vec4f,
            0.0,
            1.0,
            0.0,
            1.0,
            //resolution: vec2f,
            resolution.width as f32,
            resolution.height as f32,
            // translation: vec2f,
            resolution.width as f32 / 2.0,
            resolution.height as f32 / 2.0,
        ];
        self.queue.write_buffer(
            &uniform_buffer,
            0,
            &uniforms
                .iter()
                .flat_map(|entry| entry.to_ne_bytes())
                .collect::<Vec<u8>>(),
        );

        // Vertex buffer
        let f_char_vertices: Vec<f32> = vec![
            0.0, 0.0, 30.0, 0.0, 0.0, 150.0, 30.0, 150.0, // left column
            30.0, 0.0, 100.0, 0.0, 30.0, 30.0, 100.0, 30.0, // top rung
            30.0, 60.0, 70.0, 60.0, 30.0, 90.0, 70.0, 90.0, // middle rung
        ];
        let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            size: (size_of::<f32>() * f_char_vertices.len()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(
            &vertex_buffer,
            0,
            &f_char_vertices
                .iter()
                .flat_map(|entry| entry.to_ne_bytes())
                .collect::<Vec<u8>>(),
        );

        // Vertex indices
        let f_char_indices: Vec<u32> = vec![
            0, 1, 2, 2, 1, 3, // left column
            4, 5, 6, 6, 5, 7, // top rung
            8, 9, 10, 10, 9, 11, // middle rung
        ];
        let index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex indices"),
            size: (size_of::<f32>() * f_char_indices.len()) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(
            &index_buffer,
            0,
            &f_char_indices
                .iter()
                .flat_map(|entry| entry.to_ne_bytes())
                .collect::<Vec<u8>>(),
        );

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bind group"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniforms"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None, // use whole buffer
                }),
            }],
        });

        // Pipeline
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let swapchain_capabilities = self.surface.get_capabilities(&self.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("render_pipeline_descriptor"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs_main"),
                    buffers: &[VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(swapchain_format.into())],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..wgpu::PrimitiveState::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swap-chain texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..f_char_indices.len() as u32, 0, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
