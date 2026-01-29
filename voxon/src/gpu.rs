use std::{borrow::Cow, f32::consts::PI, sync::Arc};

use graphic::{camera::Camera, identity_matrix, transform::translate};
use lina::{m, matrix::Matrix, v, vector::Vector};
use quaternion::Quaternion;
use wgpu::{
    Adapter, BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer,
    BufferBinding, BufferUsages, DepthBiasState, DepthStencilState, Device, Face, Operations,
    Queue, RenderPassDepthStencilAttachment, RenderPipeline, StencilState, Surface,
    TextureDescriptor, TextureUsages, VertexAttribute, VertexBufferLayout, util::align_to,
};
use winit::{dpi::PhysicalSize, window::Window};

pub struct Wgpu {
    pub inner_size: PhysicalSize<u32>,
    pub adapter: Adapter,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub render_pipeline: RenderPipeline,
    pub entities: Vec<Entity>,
    pub global_uniforms: (Buffer, BindGroup),
    pub entity_uniforms: (Buffer, BindGroup),
}

pub struct Vertex {
    position: Vector<f32, 4>,
    normal: Vector<f32, 3>,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

pub struct Entity {
    // Mesh data
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    index_count: usize,
    // Transformation data
    uniform_offset: wgpu::DynamicOffset,
    world_matrix: Matrix<f32, 4, 4>,
    normal_matrix: Matrix<f32, 3, 3>,
}

// The cube center is at (0, 0, 0) and has a dimensions
// of 2.
// Return a pair of vertices and their indexes.
fn generate_cube() -> Mesh {
    // Vertex buffer
    #[rustfmt::skip]
    let vertex_positions: Vec<Vector<f32, 4>> = vec![
        v![-1.0, -1.0, 1.0, 1.0], // 0
        v![1.0, -1.0, 1.0, 1.0], // 1
        v![1.0, 1.0, 1.0, 1.0], // 2
        v![-1.0, 1.0, 1.0, 1.0], // 3
        v![-1.0, -1.0, -1.0, 1.0], // 4
        v![1.0, -1.0, -1.0, 1.0], // 5
        v![1.0, 1.0, -1.0, 1.0], // 6
        v![-1.0, 1.0, -1.0, 1.0], // 7
    ];
    // The normal will be the same for each vertex as it's position,
    // normalized.
    let vertices = vertex_positions
        .iter()
        .map(|position| Vertex {
            position: *position,
            normal: position.xyz().unwrap().normalized(),
        })
        .collect();

    // Vertex indices
    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
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

    Mesh { vertices, indices }
}

// A 2x2 big plane centered a the origo,
// laying on the XZ plane.
fn generate_plane() -> Mesh {
    // Vertex buffer
    #[rustfmt::skip]
    let vertex_positions: Vec<Vector<f32, 4>> = vec![
        v![-1.0, 0.0, 1.0, 1.0], // 0
        v![1.0, 0.0, 1.0, 1.0], // 1
        v![1.0, 0.0, -1.0, 1.0], // 2
        v![-1.0, 0.0, -1.0, 1.0], // 3
    ];
    // The normal will be the same for each vertex, up.
    let vertices = vertex_positions
        .iter()
        .map(|position| Vertex {
            position: *position,
            normal: v![0.0, 1.0, 0.0],
        })
        .collect();

    // Vertex indices
    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0, 1, 3,
        3, 1, 2  
    ];

    Mesh { vertices, indices }
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

        // CUBE
        let cube_mesh = generate_cube();
        let cube_vertex_data = cube_mesh
            .vertices
            .iter()
            .flat_map(|entry| {
                entry
                    .position
                    .as_slice()
                    .iter()
                    .chain(entry.normal.as_slice().iter().chain([&0.0]))
                    .flat_map(|value| value.to_le_bytes())
            })
            .collect::<Vec<u8>>();

        let cube_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_vertex_buffer"),
            size: cube_vertex_data.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&cube_vertex_buffer, 0, &cube_vertex_data);

        let cube_index_data = cube_mesh
            .indices
            .iter()
            .flat_map(|index| index.to_le_bytes())
            .collect::<Vec<_>>();
        let cube_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_index_buffer"),
            size: cube_index_data.len() as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&cube_index_buffer, 0, &cube_index_data);

        // PLANE
        let plane_mesh = generate_plane();
        let plane_vertex_data = plane_mesh
            .vertices
            .iter()
            .flat_map(|entry| {
                entry
                    .position
                    .as_slice()
                    .iter()
                    .chain(entry.normal.as_slice().iter().chain([&0.0]))
                    .flat_map(|value| value.to_le_bytes())
            })
            .collect::<Vec<u8>>();

        let plane_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("plane_vertex_buffer"),
            size: plane_vertex_data.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&plane_vertex_buffer, 0, &plane_vertex_data);

        let plane_index_data = plane_mesh
            .indices
            .iter()
            .flat_map(|index| index.to_le_bytes())
            .collect::<Vec<_>>();
        let plane_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("plane_index_buffer"),
            size: plane_index_data.len() as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&plane_index_buffer, 0, &plane_index_data);

        let entity_uniform_size = (16 + 16) * 4;
        let entity_uniform_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };

        let entities = {
            [
                Entity {
                    vertex_buffer: cube_vertex_buffer,
                    index_buffer: cube_index_buffer,
                    index_format: wgpu::IndexFormat::Uint32,
                    index_count: cube_mesh.indices.len(),
                    world_matrix: identity_matrix(),
                    normal_matrix: Matrix::<f32, 3, 3>::from_value(0.0),
                    uniform_offset: 0,
                },
                Entity {
                    vertex_buffer: plane_vertex_buffer,
                    index_buffer: plane_index_buffer,
                    index_format: wgpu::IndexFormat::Uint32,
                    index_count: plane_mesh.indices.len(),
                    world_matrix: graphic::transform::translate(0.0, -1.0, 0.0)
                        * graphic::transform::scale(3.0, 1.0, 3.0),
                    normal_matrix: m![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],],
                    uniform_offset: entity_uniform_alignment as u32,
                },
            ]
            .into_iter()
            .collect::<Vec<Entity>>()
        };

        // Preparing for rendering

        // Bind group layout
        let global_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        // Uniform buffer
        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("uniforms"),
                // uniforms have to be padded to a multiple of 8
                #[allow(clippy::identity_op)] // for clearer explanation
                size: (16 + 4 + 4 + 3 + 1 + 3 + 1) * 4, // (view projection matrix + light color + light position + view position + shininess + light direction + limit) * float size + padding
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("global_uniforms"),
            layout: &global_uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &global_uniform_buffer,
                    offset: 0,
                    size: None, // use whole buffer
                }),
            }],
        });

        let global_uniforms = (global_uniform_buffer, bind_group);

        let entity_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity uniform buffer"),
            size: entities.len() as u64 * entity_uniform_alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let entity_uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Local bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(entity_uniform_size), // (world matrix + normal matrix) * float size, no padding needed
                    },
                    count: None,
                }],
            });

        let entity_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Entity bind group"),
            layout: &entity_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &entity_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(entity_uniform_size),
                }),
            }],
        });
        let entity_uniforms = (entity_uniform_buffer, entity_bind_group);

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[
                &global_uniform_bind_group_layout,
                &entity_uniform_bind_group_layout,
            ],
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
                    array_stride: (4 + 3 + 1) * 4, // (4 floats for position + 3 floats for normal + 1 padding) * f32 byte count
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // position
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        // normal
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 16,
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

        Wgpu {
            inner_size,
            adapter,
            surface,
            device,
            queue,
            render_pipeline,
            entities,
            global_uniforms,
            entity_uniforms,
        }
    }

    pub fn render(
        &mut self,
        camera: &Camera,
        delta_t: std::time::Duration,
        cube_delta_t: &mut std::time::Duration,
    ) {
        // World simulation.
        // It will not be part of the render pipeline later on.
        // Only temporarily for now.

        // For now the cube transformations are hacked in here.
        let cube_full_rotation_time = std::time::Duration::from_secs(10);
        *cube_delta_t = cube_delta_t.saturating_add(delta_t);
        if *cube_delta_t > cube_full_rotation_time {
            *cube_delta_t = cube_delta_t.saturating_sub(cube_full_rotation_time);
        }

        // for quick rotation checks
        #[allow(unused_variables)]
        let rotate_y: Matrix<f32, 4, 4> = Quaternion::<f32>::new_unit(
            2.0 * PI
                * (cube_delta_t.as_millis() as f32 / cube_full_rotation_time.as_millis() as f32),
            v![0.0, 1.0, 0.0],
        )
        .into();

        // center the cube at the origo
        #[allow(unused_variables)]
        let translate = translate(-1.0, -1.0, -1.0);
        let cube_world_matrix = rotate_y;

        let cube_normal_matrix = {
            let mut matrix = Matrix::<f32, 3, 3>::new();

            matrix[(0, 0)] = cube_world_matrix[(0, 0)];
            matrix[(0, 1)] = cube_world_matrix[(0, 1)];
            matrix[(0, 2)] = cube_world_matrix[(0, 2)];

            matrix[(1, 0)] = cube_world_matrix[(1, 0)];
            matrix[(1, 1)] = cube_world_matrix[(1, 1)];
            matrix[(1, 2)] = cube_world_matrix[(1, 2)];

            matrix[(2, 0)] = cube_world_matrix[(2, 0)];
            matrix[(2, 1)] = cube_world_matrix[(2, 1)];
            matrix[(2, 2)] = cube_world_matrix[(2, 2)];

            // Adjoint is better as it always exists
            // , unlike the inverse. The only difference
            // is that the inverse is the adjoint divided by
            // the determinant.
            // So there is a scaling issue, but normals have
            // be renormalized later anyways.
            // Normal matrix would need to be transposed,
            // but WGPU already expects matrices in row major form
            // and we work with column major form.
            // So by omitting transposition on our normal matrix in
            // column major form, we provide WGPU with the transposed
            // in row major form.
            matrix.adjoint()
        };
        // a little quick heck for now, just to illustrate refactoring goals
        // but it isn't used yet
        self.entities[0].world_matrix = cube_world_matrix;
        self.entities[0].normal_matrix = cube_normal_matrix;

        // RENDER

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

        for entity in &self.entities {
            let padded_flattened_normal_matrix = [
                entity.normal_matrix[(0, 0)],
                entity.normal_matrix[(0, 1)],
                entity.normal_matrix[(0, 2)],
                0.0,
                entity.normal_matrix[(1, 0)],
                entity.normal_matrix[(1, 1)],
                entity.normal_matrix[(1, 2)],
                0.0,
                entity.normal_matrix[(2, 0)],
                entity.normal_matrix[(2, 1)],
                entity.normal_matrix[(2, 2)],
                0.0,
            ];

            let gpu_entity_bytes = entity
                .world_matrix
                .transpose()
                .as_slices()
                .iter()
                .flatten()
                .flat_map(|entry| entry.to_le_bytes())
                .chain(
                    padded_flattened_normal_matrix
                        .as_slice()
                        .iter()
                        .flat_map(|entry| entry.to_le_bytes()),
                )
                .collect::<Vec<u8>>();

            self.queue.write_buffer(
                &self.entity_uniforms.0,
                entity.uniform_offset as wgpu::BufferAddress,
                &gpu_entity_bytes,
            );
        }

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

            // the camera matrix
            let look_at = camera.as_transform_matrix();
            // view matrix
            let view_matrix = look_at;

            let aspect_ratio = self.inner_size.width as f32 / self.inner_size.height as f32;
            let projection_matrix = graphic::transform::perspective_proj_sym_h_fov(
                PI / 2.0,
                aspect_ratio,
                -1.0,
                -20000.0,
            );

            //  graphic::transform::scale(10.0, 30.0, 2.0) *
            // * graphic::transform::rotate_x(PI / 4.0)
            //  translate;
            let view_projection_matrix = projection_matrix * view_matrix;

            // Serialize to the gpu
            // WGPU works with row major matrices

            let view_projection_matrix = view_projection_matrix.transpose();

            // UPDATE Uniforms

            let global_uniforms = view_projection_matrix
                .as_slices()
                .iter()
                .flatten()
                .flat_map(|entry| entry.to_le_bytes())
                .chain(
                    // light color
                    [0.2f32, 1.0, 0.2, 1.0]
                        .iter()
                        .flat_map(|entry| entry.to_le_bytes()),
                )
                .chain(
                    // light position
                    // last value is padding
                    [-10.0f32, 10.0, 10.0, 0.0]
                        .iter()
                        .flat_map(|entry| entry.to_le_bytes()),
                )
                .chain(
                    // view position
                    [camera.eye()[0], camera.eye()[1], camera.eye()[2]]
                        .iter()
                        .flat_map(|entry| entry.to_le_bytes()),
                )
                // shininess
                .chain([100.0f32].iter().flat_map(|entry| entry.to_le_bytes()))
                .chain(
                    // light direction
                    ((-v![-1.0f32, 1.0, -1.0]).normalized())
                        .as_slice()
                        .iter()
                        .flat_map(|entry| entry.to_le_bytes()),
                )
                .chain(
                    [(90.0f32 * (PI / 180.0f32)).cos()]
                        .iter()
                        .flat_map(|entry| entry.to_le_bytes()),
                )
                .collect::<Vec<u8>>();

            self.queue
                .write_buffer(&self.global_uniforms.0, 0, &global_uniforms);
            render_pass.set_bind_group(0, &self.global_uniforms.1, &[]);

            // entities
            for entity in &self.entities {
                render_pass.set_bind_group(1, &self.entity_uniforms.1, &[entity.uniform_offset]);
                render_pass.set_index_buffer(entity.index_buffer.slice(..), entity.index_format);
                render_pass.set_vertex_buffer(0, entity.vertex_buffer.slice(..));
                render_pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
