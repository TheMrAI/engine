use std::{borrow::Cow, f32::consts::PI, sync::Arc};

use graphic::{camera::Camera, transform::rotate_y};
use wgpu::{
    Adapter, BindGroup, BindGroupEntry, Buffer, BufferBinding, BufferUsages, DepthBiasState,
    DepthStencilState, Device, Face, Operations, Queue, RenderPassDepthStencilAttachment,
    RenderPipeline, StencilState, Surface, TextureDescriptor, TextureUsages, VertexAttribute,
    VertexBufferLayout,
};
use winit::{dpi::PhysicalSize, window::Window};

pub struct Wgpu {
    pub inner_size: PhysicalSize<u32>,
    pub adapter: Adapter,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub vertex_count: u32,
    pub object_data: (Buffer, BindGroup),
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
        println!("Prepared device: {device:?}",);

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
        #[rustfmt::skip]
        let cube_vertices: Vec<f32> = vec![
            0.0, 0.0, 1.0, // 0
            1.0, 0.0, 1.0, // 1
            1.0, 1.0, 1.0, // 2
            0.0, 1.0, 1.0, // 3
            0.0, 0.0, 0.0, // 4
            1.0, 0.0, 0.0, // 5
            1.0, 1.0, 0.0, // 6
            0.0, 1.0, 0.0, // 7
        ];

        // Vertex indices
        #[rustfmt::skip]
        let cube_indices: Vec<u32> = vec![
            // front face
            0, 1, 2,
            2, 3, 0,
            // back face
            5, 4, 7,
            7, 6, 5,
            // top face
            3, 2, 6,
            6, 7, 3,
            // bottom face
            4, 5, 1,
            1, 0, 4,
            // right face
            5, 6, 2,
            2, 1, 5,
            // left face
            4, 0, 3,
            3, 7, 4
        ];

        let quad_colors: Vec<u8> = vec![
            33, 188, 255, // front (light blue / Z+)
            28, 105, 168, // back (dark blue)
            5, 223, 114, // top (light green / Y+)
            23, 130, 54, // bottom (dark green)
            255, 100, 103, // right (light red / X+)
            193, 16, 7, // left (dark red)
        ];

        let vertex_data = {
            cube_indices
                .iter()
                .enumerate()
                .flat_map(|(i, index)| {
                    let start_vertex_index = (index * 3) as usize;
                    let vertex_iter = (start_vertex_index..start_vertex_index + 3)
                        .map(|vertex_index| cube_vertices[vertex_index]);

                    let start_color_index = (i / 6) * 3;
                    let color = f32::from_le_bytes([
                        quad_colors[start_color_index],
                        quad_colors[start_color_index + 1],
                        quad_colors[start_color_index + 2],
                        255,
                    ]);

                    vertex_iter.chain([color])
                })
                .collect::<Vec<f32>>()
        };

        let vertex_data = vertex_data
            .iter()
            .flat_map(|entry| entry.to_le_bytes())
            .collect::<Vec<u8>>();

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            size: vertex_data.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, &vertex_data);

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
                    array_stride: 4 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Unorm8x4,
                            offset: 12,
                            shader_location: 1,
                        },
                    ],
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
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_compare: wgpu::CompareFunction::Less,
                depth_write_enabled: true,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Preparing for rendering
        let object_data = {
            // Uniform buffer
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("uniforms"),
                // uniforms have to be padded to a multiple of 8
                size: 16 * 4_u64, // matrix * float32 + padding
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
        };

        Wgpu {
            inner_size,
            adapter,
            surface,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            vertex_count: cube_indices.len() as u32,
            object_data,
        }
    }

    pub fn render(
        &mut self,
        camera: &Camera,
        delta_t: std::time::Duration,
        cube_delta_t: &mut std::time::Duration,
    ) {
        // Create render texture
        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swap-chain texture");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = self.device.create_texture(&TextureDescriptor {
            label: Some("depth texture"),
            size: frame.texture.size(),
            mip_level_count: 1, // no extra mips, has to be 1
            sample_count: 1,    // no multisampling, so 1
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
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
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // the camera matrix
            let look_at = camera.as_transform_matrix();
            // view matrix
            let view_matrix = look_at;

            let aspect_ratio = self.inner_size.width as f32 / self.inner_size.height as f32;
            let half_width = 8.0 / 4.0;
            let half_height = half_width / aspect_ratio;

            let projection_matrix = graphic::transform::perspective_projection_symmetric_inf(
                half_width,
                half_height,
                -1.0,
            );

            // Handle error with checked add?
            // Makes little sense
            let cube_full_rotation_time = std::time::Duration::from_secs(3);
            *cube_delta_t = cube_delta_t.saturating_add(delta_t);
            if *cube_delta_t > cube_full_rotation_time {
                *cube_delta_t = cube_delta_t.saturating_sub(cube_full_rotation_time);
            }
            let rotate_y = rotate_y(
                2.0 * PI
                    * (cube_delta_t.as_millis() as f32
                        / cube_full_rotation_time.as_millis() as f32),
            );

            let view_projection_matrix = projection_matrix * view_matrix * rotate_y;
            let matrix = view_projection_matrix;

            // WGPU works with row major matrices
            let matrix = matrix.transpose();

            let uniforms = matrix
                .as_slices()
                .iter()
                .flatten()
                .flat_map(|entry| entry.to_le_bytes())
                .collect::<Vec<u8>>();

            self.queue.write_buffer(&self.object_data.0, 0, &uniforms);

            render_pass.set_bind_group(0, &self.object_data.1, &[]);
            render_pass.draw(0..self.vertex_count, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
