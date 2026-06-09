use crate::mesh::{generate_cube, generate_plane};
use graphic::camera::Camera;
use graphic::identity_matrix;
use lina::matrix::{Matrix, m};
use lina::vector::v;
use std::f32::consts::PI;

use crate::texture::{load_texture_cube, load_texture_plane};
use quaternion::Quaternion;
use std::borrow::Cow;
use std::time::Duration;
use wgpu::BindGroup;
use wgpu::Buffer;
use wgpu::RenderPipeline;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BufferBinding, BufferUsages, DepthBiasState, DepthStencilState, Face, StencilState,
    VertexAttribute, VertexBufferLayout, util::align_to,
};

#[derive(Debug)]
struct Entity {
    // Mesh data
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    index_count: usize,
    // Transformation data
    uniform_offset: wgpu::DynamicOffset,
    world_matrix: Matrix<f32, 4, 4>,
    normal_matrix: Matrix<f32, 3, 3>,
    texture_scale: f32,
}

#[derive(Debug)]
pub struct TexturedEntities {
    // Delta_t associated with the cube rotation
    // This is not used at all now. It should be removed, but
    // it is a good reminder that animations may need to be handled at one point.
    cube_delta_t: std::time::Duration,
    // Prepared render pipeline and all the necessary info for rendering the scene
    render_pipeline: RenderPipeline,
    entities: Vec<Entity>,
    global_uniforms: (Buffer, BindGroup),
    entity_uniforms: (Buffer, BindGroup),
    texture_bind_groups: Vec<BindGroup>,
}

impl TexturedEntities {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_target: wgpu::ColorTargetState,
    ) -> Self {
        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        // CUBE
        let cube_mesh = generate_cube();
        let cube_vertex_data = cube_mesh
            .vertices()
            .iter()
            .flat_map(|entry| {
                entry
                    .position()
                    .as_slice()
                    .iter()
                    .chain(entry.normal().as_slice().iter().chain([&0.0]))
                    .chain(entry.uv().as_slice().iter())
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
            .indices()
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
            .vertices()
            .iter()
            .flat_map(|entry| {
                entry
                    .position()
                    .as_slice()
                    .iter()
                    .chain(entry.normal().as_slice().iter().chain([&0.0]))
                    .chain(entry.uv().as_slice().iter())
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
            .indices()
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

        let entity_uniform_size = (16 + 16 + 1) * 4;
        let entity_uniform_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };

        let entities = {
            [
                // Cube
                Entity {
                    vertex_buffer: cube_vertex_buffer,
                    index_buffer: cube_index_buffer,
                    index_format: wgpu::IndexFormat::Uint32,
                    index_count: cube_mesh.indices().len(),
                    world_matrix: identity_matrix(),
                    normal_matrix: m![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],],
                    uniform_offset: 0,
                    texture_scale: 1.0,
                },
                // Plane
                Entity {
                    vertex_buffer: plane_vertex_buffer,
                    index_buffer: plane_index_buffer,
                    index_format: wgpu::IndexFormat::Uint32,
                    index_count: plane_mesh.indices().len(),
                    world_matrix: graphic::transform::translate(0.0, -1.0, 0.0)
                        * graphic::transform::scale(50.0, 1.0, 50.0),
                    normal_matrix: m![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],],
                    uniform_offset: entity_uniform_alignment as u32,
                    texture_scale: 50.0,
                },
            ]
            .into_iter()
            .collect::<Vec<Entity>>()
        };

        let cube_texture_view = load_texture_cube(device, queue);
        let plane_texture_view = load_texture_plane(device, queue);

        let sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor {
            label: Some("texture_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_sampler_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let cube_texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cube_texture_view),
                },
            ],
        });
        let plane_texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&plane_texture_view),
                },
            ],
        });
        let texture_bind_groups = vec![cube_texture_bind_group, plane_texture_bind_group];

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
            size: (16 + 3) * 4 + 4, // (view projection matrix + view position) * float size + padding
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let global_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        let global_uniforms = (global_uniform_buffer, global_uniform_bind_group);

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

        for entity in &entities {
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
                .chain(
                    [entity.texture_scale]
                        .iter()
                        .flat_map(|entry| entry.to_le_bytes()),
                )
                .collect::<Vec<u8>>();

            queue.write_buffer(
                &entity_uniforms.0,
                entity.uniform_offset as wgpu::BufferAddress,
                &gpu_entity_bytes,
            );
        }

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[
                &global_uniform_bind_group_layout,
                &entity_uniform_bind_group_layout,
                &texture_bind_group_layout,
            ],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline_descriptor"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: (4 + 3 + 1 + 2) * 4, // (4 floats for position + 3 floats for normal + 1 padding + 2 UV) * f32 byte count
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
                        // uv
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 32,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(color_target)],
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
            multiview_mask: None,
            cache: None,
        });

        Self {
            cube_delta_t: Duration::default(),
            render_pipeline,
            entities,
            global_uniforms,
            entity_uniforms,
            texture_bind_groups,
        }
    }

    // This has no effect, because we never update the initial buffers after the objects
    // creation.
    pub fn simulate(&mut self, delta_t: std::time::Duration) {
        // It will not be part of the render pipeline later on.
        // Only temporarily for now.

        // For now the cube transformations are hacked in here.
        let cube_full_rotation_time = std::time::Duration::from_secs(10);
        self.cube_delta_t = self.cube_delta_t.saturating_add(delta_t);
        if self.cube_delta_t > cube_full_rotation_time {
            self.cube_delta_t = self.cube_delta_t.saturating_sub(cube_full_rotation_time);
        }

        // for quick rotation checks
        #[allow(unused_variables)]
        let rotate_y: Matrix<f32, 4, 4> = Quaternion::<f32>::new_unit(
            2.0 * PI
                * (self.cube_delta_t.as_millis() as f32
                    / cube_full_rotation_time.as_millis() as f32),
            v![0.0, 1.0, 0.0],
        )
        .into();

        let cube_world_matrix = rotate_y * graphic::identity_matrix();

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

        let cube = &mut self.entities[0];
        cube.world_matrix = cube_world_matrix;
        cube.normal_matrix = cube_normal_matrix;
    }

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        camera: &Camera,
        view_projection_matrix: &Matrix<f32, 4, 4>,
    ) {
        // Serialize to the gpu
        // WGPU works with row major matrices
        let view_projection_matrix = view_projection_matrix.transpose();

        render_pass.set_pipeline(&self.render_pipeline);

        // UPDATE Uniforms
        let global_uniforms = view_projection_matrix
            .as_slices()
            .iter()
            .flatten()
            .flat_map(|entry| entry.to_le_bytes())
            .chain(
                // view position
                [camera.eye()[0], camera.eye()[1], camera.eye()[2]]
                    .iter()
                    .flat_map(|entry| entry.to_le_bytes()),
            )
            .collect::<Vec<u8>>();

        queue.write_buffer(&self.global_uniforms.0, 0, &global_uniforms);
        render_pass.set_bind_group(0, &self.global_uniforms.1, &[]);

        // entities
        for (i, entity) in self.entities.iter().enumerate() {
            render_pass.set_bind_group(1, &self.entity_uniforms.1, &[entity.uniform_offset]);
            render_pass.set_bind_group(2, &self.texture_bind_groups[i], &[]);
            render_pass.set_index_buffer(entity.index_buffer.slice(..), entity.index_format);
            render_pass.set_vertex_buffer(0, entity.vertex_buffer.slice(..));
            render_pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
        }
    }
}
