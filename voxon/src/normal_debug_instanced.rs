use graphic::camera::Camera;
use lina::matrix::Matrix;

use std::borrow::Cow;
use wgpu::BindGroup;
use wgpu::Buffer;
use wgpu::RenderPipeline;
use wgpu::{
    BindGroupEntry, BufferBinding, BufferUsages, DepthBiasState, DepthStencilState, Face,
    StencilState, VertexAttribute, VertexBufferLayout,
};

#[derive(Debug)]
pub struct InstancedNormalDebug {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    index_count: usize,
    transforms: Vec<Matrix<f32, 4, 4>>,
    // above three lines is basically Entity from non instanced case
    global_uniforms_buffer: Buffer,
    transforms_buffer: Buffer,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

impl InstancedNormalDebug {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_target: wgpu::ColorTargetState,
    ) -> Self {
        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("instanced_shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "normal_debug_instanced.wgsl"
            ))),
        });

        // Stanford dragon smooth 700k
        let stanford_dragon_smooth_700k_data =
            include_str!("../resources/meshes/stanford_dragon_smooth_700k.obj");
        let stanford_dragon_smooth_700k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_700k_data.lines().map(String::from),
            "Stanford_dragon_smooth_700k",
        );

        let stanford_dragon_smooth_700k_vertex_data = stanford_dragon_smooth_700k
            .faces()
            .iter()
            .flat_map(|face| {
                let vertices = stanford_dragon_smooth_700k.vertices();
                let normals = stanford_dragon_smooth_700k.normals();

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
        let stanford_dragon_smooth_700k_vertex_buffer =
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("stanford_dragon_s700k_vertex_buffer"),
                size: stanford_dragon_smooth_700k_vertex_data.len() as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        queue.write_buffer(
            &stanford_dragon_smooth_700k_vertex_buffer,
            0,
            &stanford_dragon_smooth_700k_vertex_data,
        );

        let stanford_dragon_smooth_700k_index_data =
            (0..stanford_dragon_smooth_700k.faces().len() as u32 * 3)
                .flat_map(|index| index.to_le_bytes())
                .collect::<Vec<_>>();

        let stanford_dragon_smooth_700k_index_buffer =
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("stanford_dragon_s700k_index_buffer"),
                size: stanford_dragon_smooth_700k_index_data.len() as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        queue.write_buffer(
            &stanford_dragon_smooth_700k_index_buffer,
            0,
            &stanford_dragon_smooth_700k_index_data,
        );

        let mut transforms = Vec::<Matrix<f32, 4, 4>>::new();
        // generate stanford dragons 700k transforms
        let mut x = -10.0;
        while x <= 10.0 {
            let mut z = -5.0;
            while z >= -25.0 {
                let mut y = 5.0;
                while y <= 15.0 {
                    transforms.push(
                        graphic::transform::translate(x, y, z)
                            * graphic::transform::scale(18.0, 18.0, 18.0),
                    );
                    y += 5.0;
                }
                z -= 5.0;
            }
            x += 5.0;
        }

        // Bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind_group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Uniform buffer
        let global_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            // uniforms have to be padded to a multiple of 8
            #[allow(clippy::identity_op)] // for clearer explanation
            size: (16 + 16) * 4, // (view matrix, view projection matrix) * float size
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // (world matrix + normal matrix) * float size, no padding needed
        let transforms_size = (16 + 12) * 4;
        let transforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("transform buffer"),
            size: transforms.len() as u64 * transforms_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &global_uniforms_buffer,
                        offset: 0,
                        size: None, // use whole buffer
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &transforms_buffer,
                        offset: 0,
                        size: None, // use whole buffer
                    }),
                },
            ],
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
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
            vertex_buffer: stanford_dragon_smooth_700k_vertex_buffer,
            index_buffer: stanford_dragon_smooth_700k_index_buffer,
            index_format: wgpu::IndexFormat::Uint32,
            index_count: stanford_dragon_smooth_700k.faces().len() * 3,
            transforms,
            global_uniforms_buffer,
            transforms_buffer,
            bind_group,
            render_pipeline,
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
        render_pass.set_pipeline(&self.render_pipeline);

        // Serialize to the gpu
        // WGPU works with row major matrices
        let transposed_view_matrix = view_matrix.transpose();
        let transposed_view_projection_matrix = view_projection_matrix.transpose();

        // UPDATE Uniforms
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
            .collect::<Vec<u8>>();
        queue.write_buffer(&self.global_uniforms_buffer, 0, &global_uniforms);

        // UPDATE transforms buffer
        let mut transform_buffer_data = Vec::<u8>::new();
        // Update entity uniforms
        for world_matrix in &self.transforms {
            let normal_matrix = {
                let view_model_matrix = *view_matrix * *world_matrix;

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

            let mut gpu_entity_bytes = world_matrix
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
            transform_buffer_data.append(&mut gpu_entity_bytes);
        }
        queue.write_buffer(&self.transforms_buffer, 0, &transform_buffer_data);

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), self.index_format);

        render_pass.draw_indexed(
            0..self.index_count as u32,
            0,
            0..self.transforms.len() as u32,
        );
    }
}
