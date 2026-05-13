use graphic::camera::Camera;
use lina::matrix::{Matrix, m};

use std::borrow::Cow;
use wgpu::BindGroup;
use wgpu::Buffer;
use wgpu::RenderPipeline;
use wgpu::{
    BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferBinding, BufferUsages,
    DepthBiasState, DepthStencilState, Face, StencilState, VertexAttribute, VertexBufferLayout,
    util::align_to,
};

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
}

pub struct NormalDebug {
    // Prepared render pipeline and all the necessary info for rendering the scene
    render_pipeline: RenderPipeline,
    entities: Vec<Entity>,
    global_uniforms: (Buffer, BindGroup),
    entity_uniforms: (Buffer, BindGroup),
}

impl NormalDebug {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_target: wgpu::ColorTargetState,
    ) -> Self {
        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("normal_debug.wgsl"))),
        });

        // SUZANNE flat 975
        let suzanne_flat975_data = include_str!("../resources/meshes/suzanne_flat_967.obj");
        let suzanne_flat975 = format::wavefront::Obj::parse(
            suzanne_flat975_data.lines().map(String::from),
            "Suzanne",
        );

        let suzanne_flat975_vertex_data = suzanne_flat975
            .faces()
            .iter()
            .flat_map(|face| {
                let vertices = suzanne_flat975.vertices();
                let normals = suzanne_flat975.normals();

                face.iter().flat_map(|vertex| {
                    let face_vertex = &vertices[vertex.vertex_index() - 1];
                    // it is possible that a mesh doesn't contain normals either
                    // may have to handle it
                    let face_normals = &normals[vertex.normal_index().unwrap() - 1];

                    face_vertex
                        .as_slice()
                        .iter()
                        .chain(face_normals.as_slice().iter().chain([&0.0]))
                        .flat_map(|value| value.to_le_bytes())
                })
            })
            .collect::<Vec<u8>>();

        // crudely convert the data into a vertex buffer by duplicating every single
        // vertex
        let suzanne_flat975_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("suzanne_vertex_buffer"),
            size: suzanne_flat975_vertex_data.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &suzanne_flat975_vertex_buffer,
            0,
            &suzanne_flat975_vertex_data,
        );

        let suzanne_flat975_index_data = (0..suzanne_flat975.faces().len() as u32 * 3)
            .flat_map(|index| index.to_le_bytes())
            .collect::<Vec<_>>();

        let suzanne_flat975_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("suzanne_index_buffer"),
            size: suzanne_flat975_index_data.len() as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &suzanne_flat975_index_buffer,
            0,
            &suzanne_flat975_index_data,
        );

        // (world matrix + normal matrix) * float size, no padding needed
        let entity_uniform_size = (16 + 16) * 4;
        let entity_uniform_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };

        let entities = {
            [
                // Suzanne flat
                Entity {
                    vertex_buffer: suzanne_flat975_vertex_buffer,
                    index_buffer: suzanne_flat975_index_buffer,
                    index_format: wgpu::IndexFormat::Uint32,
                    index_count: suzanne_flat975.faces().len() * 3,
                    world_matrix: graphic::transform::translate(0.0, 0.0, -5.0),
                    // this does nothing, as it has to be updated all the time anyways
                    normal_matrix: m![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],],
                    uniform_offset: 0,
                },
            ]
            .into_iter()
            .collect::<Vec<Entity>>()
        };

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
            size: (16 + 16) * 4, // (view matrix, view projection matrix) * float size
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
                        min_binding_size: wgpu::BufferSize::new(entity_uniform_size),
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
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline_descriptor"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: (4 + 3 + 1) * 4, // (4 floats for position + 3 floats for normal + 1) * f32 byte count
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
            render_pipeline,
            entities,
            global_uniforms,
            entity_uniforms,
        }
    }

    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        _camera: &Camera,
        view_matrix: &Matrix<f32, 4, 4>,
        view_projection_matrix: &Matrix<f32, 4, 4>,
    ) {
        // Update entity uniforms
        for entity in &self.entities {
            let normal_matrix = {
                let view_model_matrix = *view_matrix * entity.world_matrix;

                let mut matrix = Matrix::<f32, 3, 3>::new();
                matrix[(0, 0)] = view_model_matrix[(0, 0)];
                matrix[(0, 1)] = view_model_matrix[(0, 1)];
                matrix[(0, 2)] = view_model_matrix[(0, 2)];

                matrix[(1, 0)] = view_model_matrix[(1, 0)];
                matrix[(1, 1)] = view_model_matrix[(1, 1)];
                matrix[(1, 2)] = view_model_matrix[(1, 2)];

                matrix[(2, 0)] = view_model_matrix[(2, 0)];
                matrix[(2, 1)] = view_model_matrix[(2, 1)];
                matrix[(2, 2)] = view_model_matrix[(2, 2)];

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

            let padded_flattened_normal_matrix = [
                normal_matrix[(0, 0)],
                normal_matrix[(0, 1)],
                normal_matrix[(0, 2)],
                0.0,
                normal_matrix[(1, 0)],
                normal_matrix[(1, 1)],
                normal_matrix[(1, 2)],
                0.0,
                normal_matrix[(2, 0)],
                normal_matrix[(2, 1)],
                normal_matrix[(2, 2)],
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

            queue.write_buffer(
                &self.entity_uniforms.0,
                entity.uniform_offset as wgpu::BufferAddress,
                &gpu_entity_bytes,
            );
        }

        // Serialize to the gpu
        // WGPU works with row major matrices
        let view_matrix = view_matrix.transpose();
        let view_projection_matrix = view_projection_matrix.transpose();

        render_pass.set_pipeline(&self.render_pipeline);

        // UPDATE Uniforms
        let global_uniforms = view_matrix
            .as_slices()
            .iter()
            .flatten()
            .flat_map(|entry| entry.to_le_bytes())
            .chain(
                view_projection_matrix
                    .as_slices()
                    .iter()
                    .flatten()
                    .flat_map(|entry| entry.to_le_bytes()),
            )
            .collect::<Vec<u8>>();

        queue.write_buffer(&self.global_uniforms.0, 0, &global_uniforms);
        render_pass.set_bind_group(0, &self.global_uniforms.1, &[]);

        // entities
        for (_i, entity) in self.entities.iter().enumerate() {
            render_pass.set_bind_group(1, &self.entity_uniforms.1, &[entity.uniform_offset]);
            render_pass.set_index_buffer(entity.index_buffer.slice(..), entity.index_format);
            render_pass.set_vertex_buffer(0, entity.vertex_buffer.slice(..));
            render_pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
        }
    }
}
