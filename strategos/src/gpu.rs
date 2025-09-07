use std::{borrow::Cow, f32::consts::PI, sync::Arc};

use lina::{matrix::Matrix, v, vector::Vector};
use wgpu::{
    Adapter, BindGroup, BindGroupEntry, Buffer, BufferBinding, BufferUsages, DepthBiasState,
    DepthStencilState, Device, Face, Operations, Queue, RenderPassDepthStencilAttachment,
    RenderPipeline, StencilState, Surface, TextureDescriptor, TextureUsages, VertexAttribute,
    VertexBufferLayout,
};
use winit::{dpi::PhysicalSize, window::Window};

pub fn inverse(a: Matrix<f32, 4, 4>) -> Matrix<f32, 4, 4> {
    let mut inverse = Matrix::<f32, 4, 4>::from_value(0.0);

    let m00 = a[(0, 0)];
    let m01 = a[(0, 1)];
    let m02 = a[(0, 2)];
    let m03 = a[(0, 3)];
    let m10 = a[(1, 0)];
    let m11 = a[(1, 1)];
    let m12 = a[(1, 2)];
    let m13 = a[(1, 3)];
    let m20 = a[(2, 0)];
    let m21 = a[(2, 1)];
    let m22 = a[(2, 2)];
    let m23 = a[(2, 3)];
    let m30 = a[(3, 0)];
    let m31 = a[(3, 1)];
    let m32 = a[(3, 2)];
    let m33 = a[(3, 3)];

    let tmp0 = m22 * m33;
    let tmp1 = m32 * m23;
    let tmp2 = m12 * m33;
    let tmp3 = m32 * m13;
    let tmp4 = m12 * m23;
    let tmp5 = m22 * m13;
    let tmp6 = m02 * m33;
    let tmp7 = m32 * m03;
    let tmp8 = m02 * m23;
    let tmp9 = m22 * m03;
    let tmp10 = m02 * m13;
    let tmp11 = m12 * m03;
    let tmp12 = m20 * m31;
    let tmp13 = m30 * m21;
    let tmp14 = m10 * m31;
    let tmp15 = m30 * m11;
    let tmp16 = m10 * m21;
    let tmp17 = m20 * m11;
    let tmp18 = m00 * m31;
    let tmp19 = m30 * m01;
    let tmp20 = m00 * m21;
    let tmp21 = m20 * m01;
    let tmp22 = m00 * m11;
    let tmp23 = m10 * m01;

    let t0 = (tmp0 * m11 + tmp3 * m21 + tmp4 * m31) - (tmp1 * m11 + tmp2 * m21 + tmp5 * m31);
    let t1 = (tmp1 * m01 + tmp6 * m21 + tmp9 * m31) - (tmp0 * m01 + tmp7 * m21 + tmp8 * m31);
    let t2 = (tmp2 * m01 + tmp7 * m11 + tmp10 * m31) - (tmp3 * m01 + tmp6 * m11 + tmp11 * m31);
    let t3 = (tmp5 * m01 + tmp8 * m11 + tmp11 * m21) - (tmp4 * m01 + tmp9 * m11 + tmp10 * m21);

    let d = 1.0 / (m00 * t0 + m10 * t1 + m20 * t2 + m30 * t3);

    inverse[(0, 0)] = d * t0;
    inverse[(0, 1)] = d * t1;
    inverse[(0, 2)] = d * t2;
    inverse[(0, 3)] = d * t3;

    inverse[(1, 0)] =
        d * ((tmp1 * m10 + tmp2 * m20 + tmp5 * m30) - (tmp0 * m10 + tmp3 * m20 + tmp4 * m30));
    inverse[(1, 1)] =
        d * ((tmp0 * m00 + tmp7 * m20 + tmp8 * m30) - (tmp1 * m00 + tmp6 * m20 + tmp9 * m30));
    inverse[(1, 2)] =
        d * ((tmp3 * m00 + tmp6 * m10 + tmp11 * m30) - (tmp2 * m00 + tmp7 * m10 + tmp10 * m30));
    inverse[(1, 3)] =
        d * ((tmp4 * m00 + tmp9 * m10 + tmp10 * m20) - (tmp5 * m00 + tmp8 * m10 + tmp11 * m20));

    inverse[(2, 0)] =
        d * ((tmp12 * m13 + tmp15 * m23 + tmp16 * m33) - (tmp13 * m13 + tmp14 * m23 + tmp17 * m33));
    inverse[(2, 1)] =
        d * ((tmp13 * m03 + tmp18 * m23 + tmp21 * m33) - (tmp12 * m03 + tmp19 * m23 + tmp20 * m33));
    inverse[(2, 2)] =
        d * ((tmp14 * m03 + tmp19 * m13 + tmp22 * m33) - (tmp15 * m03 + tmp18 * m13 + tmp23 * m33));
    inverse[(2, 3)] =
        d * ((tmp17 * m03 + tmp20 * m13 + tmp23 * m23) - (tmp16 * m03 + tmp21 * m13 + tmp22 * m23));

    inverse[(3, 0)] =
        d * ((tmp14 * m22 + tmp17 * m32 + tmp13 * m12) - (tmp16 * m32 + tmp12 * m12 + tmp15 * m22));
    inverse[(3, 1)] =
        d * ((tmp20 * m32 + tmp12 * m02 + tmp19 * m22) - (tmp18 * m22 + tmp21 * m32 + tmp13 * m02));
    inverse[(3, 2)] =
        d * ((tmp18 * m12 + tmp23 * m32 + tmp15 * m02) - (tmp22 * m32 + tmp14 * m02 + tmp19 * m12));
    inverse[(3, 3)] =
        d * ((tmp22 * m22 + tmp16 * m02 + tmp21 * m12) - (tmp20 * m12 + tmp23 * m22 + tmp17 * m02));
    return inverse;
}

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
        #[rustfmt::skip]
        let f_char_vertices: Vec<f32> = vec![
            // left column
            -50.0,  75.0,  15.0,
            -20.0,  75.0,  15.0,
            -50.0, -75.0,  15.0,
            -20.0, -75.0,  15.0,
           // top rung
            -20.0,  75.0,  15.0,
             50.0,  75.0,  15.0,
            -20.0,  45.0,  15.0,
             50.0,  45.0,  15.0,
           // middle rung
            -20.0,  15.0,  15.0,
             20.0,  15.0,  15.0,
            -20.0, -15.0,  15.0,
             20.0, -15.0,  15.0,
           // left column back
            -50.0,  75.0, -15.0,
            -20.0,  75.0, -15.0,
            -50.0, -75.0, -15.0,
            -20.0, -75.0, -15.0,
           // top rung back
            -20.0,  75.0, -15.0,
             50.0,  75.0, -15.0,
            -20.0,  45.0, -15.0,
             50.0,  45.0, -15.0,
           // middle rung back
            -20.0,  15.0, -15.0,
             20.0,  15.0, -15.0,
            -20.0, -15.0, -15.0,
             20.0, -15.0, -15.0,
        ];

        // Vertex indices
        let f_char_indices: Vec<u32> = vec![
            0, 2, 1, 2, 3, 1, // left column
            4, 6, 5, 6, 7, 5, // top run
            8, 10, 9, 10, 11, 9, // middle run
            12, 13, 14, 14, 13, 15, // left column back
            16, 17, 18, 18, 17, 19, // top run back
            20, 21, 22, 22, 21, 23, // middle run back
            0, 5, 12, 12, 5, 17, // top
            5, 7, 17, 17, 7, 19, // top rung right
            6, 18, 7, 18, 19, 7, // top rung bottom
            6, 8, 18, 18, 8, 20, // between top and middle rung
            8, 9, 20, 20, 9, 21, // middle rung top
            9, 11, 21, 21, 11, 23, // middle rung right
            10, 22, 11, 22, 23, 11, // middle rung bottom
            10, 3, 22, 22, 3, 15, // stem right
            2, 14, 3, 14, 15, 3, // bottom
            0, 12, 2, 12, 14, 2, // left
        ];
        // Each vertex index corresponds to a vertex to be used which is
        // more than the number of vertices we have.
        let vertex_count = f_char_indices.len() as u32;

        let quad_colors: Vec<u8> = vec![
            200, 70, 120, // left column front
            200, 70, 120, // top rung front
            200, 70, 120, // middle rung front
            80, 70, 200, // left column back
            80, 70, 200, // top rung back
            80, 70, 200, // middle rung back
            70, 200, 210, // top
            160, 160, 220, // top rung right
            90, 130, 110, // top rung bottom
            200, 200, 70, // between top and middle rung
            210, 100, 70, // middle rung top
            210, 160, 70, // middle rung right
            70, 180, 210, // middle rung bottom
            100, 70, 210, // stem right
            76, 210, 100, // bottom
            140, 210, 80, // left
        ];

        let vertex_data = {
            f_char_indices
                .iter()
                .enumerate()
                .flat_map(|(i, index)| {
                    let start_vertex_index = (index * 3) as usize;
                    let vertex_iter = (start_vertex_index..start_vertex_index + 3)
                        .map(|vertex_index| f_char_vertices[vertex_index]);

                    let start_color_index = (i / 6 | 0) as usize * 3;
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
            vertex_count,
            object_data,
        }
    }

    pub fn render(&mut self, camera_eye: Vector<f32, 3>) {
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

            // the up vector for the camera
            let up = v![0.0, 1.0, 0.0];
            // the camera matrix
            let camera = graphic::look_at(camera_eye, v![0.0, 0.0, -120.0], up);
            // view matrix
            let view_matrix = inverse(camera);

            let projecton_matrix = graphic::perspective_projection(
                PI / 2.0, // PI / 2.0 rad => 90 degrees
                self.inner_size.width as f32 / self.inner_size.height as f32,
                1.0,
                2000.0,
            );
            let view_projection_matrix = projecton_matrix * view_matrix;

            let translation = graphic::translate(0.0, 0.0, -120.0);
            let rotation_on_y = graphic::rotate_y(-PI / 4.0);
            let rotation_on_z = graphic::rotate_z(-PI / 4.0);
            let scaling = graphic::scale(1.0, 1.0, 1.0);
            // move the origin of the 'F' into the origo
            let translate_origin = graphic::translate(-50.0, -75.0, 0.0);
            let matrix = view_projection_matrix
                * ((((translation * rotation_on_z) * rotation_on_y) * scaling) * translate_origin);

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
