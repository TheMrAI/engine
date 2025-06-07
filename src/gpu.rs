use std::{borrow::Cow, sync::Arc};

use wgpu::{
    Adapter, BindGroup, BindGroupEntry, Buffer, BufferBinding, BufferUsages, Device, Queue,
    RenderPipeline, Surface, VertexAttribute, VertexBufferLayout,
};
use winit::{dpi::PhysicalSize, window::Window};

// Notice that all transformation matrices are transposed compared
// to how they would appear in an algebra book.
#[rustfmt::skip]
pub fn translate(translate_x: f32, translate_y: f32) -> Vec<f32> {
    vec![
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        translate_x, translate_y, 1.0
    ]
}

#[rustfmt::skip]
pub fn rotate(rad_angle: f32) -> Vec<f32> {
    let cosine = rad_angle.cos();
    let sine = rad_angle.sin();
    vec![
        cosine, sine, 0.0,
        -sine, cosine, 0.0,
        0.0, 0.0, 1.0
    ]
}

#[rustfmt::skip]
pub fn scale(scale_x: f32, scale_y: f32) -> Vec<f32> {
    vec![
        scale_x, 0.0, 0.0,
        0.0, scale_y, 0.0,
        0.0, 0.0, 1.0
    ]
}

#[rustfmt::skip]
pub fn identity_matrix() -> Vec<f32> {
    vec![
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0
    ]
}

#[rustfmt::skip]
#[allow(clippy::all)]
pub fn multiply(lhs: &[f32], rhs: &[f32]) -> Vec<f32> {
    debug_assert!(lhs.len() == rhs.len());

    let a00 = lhs[0 * 3 + 0];
    let a01 = lhs[0 * 3 + 1];
    let a02 = lhs[0 * 3 + 2];
    let a10 = lhs[1 * 3 + 0];
    let a11 = lhs[1 * 3 + 1];
    let a12 = lhs[1 * 3 + 2];
    let a20 = lhs[2 * 3 + 0];
    let a21 = lhs[2 * 3 + 1];
    let a22 = lhs[2 * 3 + 2];
    let b00 = rhs[0 * 3 + 0];
    let b01 = rhs[0 * 3 + 1];
    let b02 = rhs[0 * 3 + 2];
    let b10 = rhs[1 * 3 + 0];
    let b11 = rhs[1 * 3 + 1];
    let b12 = rhs[1 * 3 + 2];
    let b20 = rhs[2 * 3 + 0];
    let b21 = rhs[2 * 3 + 1];
    let b22 = rhs[2 * 3 + 2];
 
    vec![
      b00 * a00 + b01 * a10 + b02 * a20,
      b00 * a01 + b01 * a11 + b02 * a21,
      b00 * a02 + b01 * a12 + b02 * a22,
      b10 * a00 + b11 * a10 + b12 * a20,
      b10 * a01 + b11 * a11 + b12 * a21,
      b10 * a02 + b11 * a12 + b12 * a22,
      b20 * a00 + b21 * a10 + b22 * a20,
      b20 * a01 + b21 * a11 + b22 * a21,
      b20 * a02 + b21 * a12 + b22 * a22,
    ]
}

pub struct Wgpu {
    pub inner_size: PhysicalSize<u32>,
    pub adapter: Adapter,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub object_datas: Vec<(Buffer, BindGroup)>,
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

        // Vertex buffer
        let f_char_vertices: Vec<f32> = vec![
            0.0, 0.0, 30.0, 0.0, 0.0, 150.0, 30.0, 150.0, // left column
            30.0, 0.0, 100.0, 0.0, 30.0, 30.0, 100.0, 30.0, // top rung
            30.0, 60.0, 70.0, 60.0, 30.0, 90.0, 70.0, 90.0, // middle rung
        ];
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            size: (size_of::<f32>() * f_char_vertices.len()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
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
        let index_count = f_char_indices.len() as u32;
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex indices"),
            size: (size_of::<f32>() * f_char_indices.len()) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &index_buffer,
            0,
            &f_char_indices
                .iter()
                .flat_map(|entry| entry.to_ne_bytes())
                .collect::<Vec<u8>>(),
        );

        // Bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind_group"),
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

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline_descriptor"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
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
                module: &shader,
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

        // Preparing for rendering
        let object_datas = (0..5)
            .map(|_| {
                // Uniform buffer
                let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("uniforms"),
                    // uniforms have to be padded to a multiple of 8
                    size: (4 + 2 + 2 + 12) * 4_u64, // (color + resolution + matrix) * float32 + padding
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                // Create bind group
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

                (uniform_buffer, bind_group)
            })
            .collect();

        Wgpu {
            inner_size,
            adapter,
            surface,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            object_datas,
        }
    }

    pub fn render(&mut self) {
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
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            let translation = translate(82f32, 20f32);
            let rotation = rotate(0.0872665);
            let scaling = scale(0.9, 0.8);
            let matrix = multiply(&multiply(&translation, &rotation), &scaling);

            let mut total_transform = identity_matrix();

            for object_data in &self.object_datas {
                let uniforms: Vec<f32> = {
                    let mut uniforms = vec![
                        //color: vec4f,
                        0.0,
                        1.0,
                        0.0,
                        1.0,
                        //resolution: vec2f,
                        self.inner_size.width as f32,
                        self.inner_size.height as f32,
                        // padding before matrix
                        0.0,
                        0.0, //matrix
                    ];
                    let mut padded_matrix = total_transform
                        .chunks(3)
                        .zip([0.0, 0.0, 0.0].iter())
                        .flat_map(|(row, padding)| vec![row[0], row[1], row[2], *padding])
                        .collect::<Vec<f32>>();
                    uniforms.append(&mut padded_matrix);
                    uniforms
                };

                let uniforms = uniforms
                    .iter()
                    .flat_map(|entry| entry.to_ne_bytes())
                    .collect::<Vec<u8>>();

                self.queue.write_buffer(
                    &object_data.0,
                    0,
                    &uniforms
                        .iter()
                        .flat_map(|entry| entry.to_ne_bytes())
                        .collect::<Vec<u8>>(),
                );

                render_pass.set_bind_group(0, &object_data.1, &[]);
                render_pass.draw_indexed(0..self.index_count, 0, 0..1);

                total_transform = multiply(&total_transform, &matrix);
            }
        }
        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
