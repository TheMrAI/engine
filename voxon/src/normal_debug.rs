use graphic::camera::Camera;
use lina::matrix::Matrix;

use std::borrow::Cow;
use wgpu::RenderPipeline;
use wgpu::{
    BindGroupEntry, BufferBinding, BufferUsages, DepthBiasState, DepthStencilState, Face,
    StencilState, util::align_to,
};

#[derive(Debug)]
struct Entity {
    model_matrix: Matrix<f32, 4, 4>,
}

fn append_model_to_vertex_buffer(
    vertex_buffer_data: &mut Vec<u8>,
    vertex_buffer_offsets: &mut Vec<u32>,
    vertex_total: &mut u32,
    vertex_counts: &mut Vec<u32>,
    object: &format::wavefront::Obj,
) {
    let mut vertex_data = object
        .faces()
        .iter()
        .flat_map(|face| {
            let vertices = object.vertices();
            let normals = object.normals();

            face.iter().flat_map(|vertex| {
                let face_vertex = &vertices[vertex.vertex_index() - 1];
                // it is possible that a mesh doesn't contain normals either
                // may have to handle it
                let face_normal = &normals[vertex.normal_index().unwrap() - 1];

                face_vertex
                    .as_slice()
                    .iter()
                    .chain(face_normal.as_slice().iter().chain([&0.0]))
                    .flat_map(|value| value.to_le_bytes())
            })
        })
        .collect::<Vec<u8>>();

    vertex_buffer_data.append(&mut vertex_data);
    vertex_buffer_offsets.push(*vertex_total);
    vertex_counts.push(object.faces().len() as u32 * 3);
    *vertex_total += vertex_counts.last().unwrap();
}

#[derive(Debug)]
pub struct NormalDebug {
    // Non-instanced entities
    // Prepared render pipeline and all the necessary info for rendering the scene
    render_pipeline: RenderPipeline,
    entities: Vec<Entity>,
    vertex_counts: Vec<u32>,
    entity_uniform_alignment: u32,
    bind_group: wgpu::BindGroup,
    global_uniform_buffer: wgpu::Buffer,
    entity_uniform_buffer: wgpu::Buffer,
    vertex_buffer_offsets: Vec<u32>,
}

impl NormalDebug {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_target: wgpu::ColorTargetState,
    ) -> Self {
        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("normal_debug"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("normal_debug.wgsl"))),
        });

        let mut vertex_buffer_offsets = Vec::<u32>::new();
        let mut vertex_total = 0u32;
        let mut vertex_buffer_data = Vec::<u8>::new();
        let mut vertex_counts = Vec::<u32>::new();

        // SUZANNE flat 967
        let suzanne_flat_967_data = include_str!("../resources/meshes/suzanne_flat_967.obj");
        let suzanne_flat_967 = format::wavefront::Obj::parse(
            suzanne_flat_967_data.lines().map(String::from),
            "Suzanne_flat_967",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &suzanne_flat_967,
        );

        // SUZANNE flat 967 messed up normals
        let suzanne_flat_967_messed_up_normals_data =
            include_str!("../resources/meshes/suzanne_flat_967_messed_up_normals.obj");
        let suzanne_flat_967_messed_up_normals = format::wavefront::Obj::parse(
            suzanne_flat_967_messed_up_normals_data
                .lines()
                .map(String::from),
            "Suzanne_flat_967_messed_up_normals",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &suzanne_flat_967_messed_up_normals,
        );

        // SUZANNE smooth 967
        let suzanne_smooth_967_data = include_str!("../resources/meshes/suzanne_smooth_967.obj");
        let suzanne_smooth_967 = format::wavefront::Obj::parse(
            suzanne_smooth_967_data.lines().map(String::from),
            "Suzanne_smooth_967",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &suzanne_smooth_967,
        );

        // SUZANNE smooth 967 messed up normals
        let suzanne_smooth_967_messed_up_normals_data =
            include_str!("../resources/meshes/suzanne_smooth_967_messed_up_normals.obj");
        let suzanne_smooth_967_messed_up_normals = format::wavefront::Obj::parse(
            suzanne_smooth_967_messed_up_normals_data
                .lines()
                .map(String::from),
            "Suzanne_smooth_967_messed_up_normals",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &suzanne_smooth_967_messed_up_normals,
        );

        // Utah teapot flat 7k
        let utah_flat_7k_data = include_str!("../resources/meshes/utah_teapot_flat_7k.obj");
        let utah_flat_7k = format::wavefront::Obj::parse(
            utah_flat_7k_data.lines().map(String::from),
            "Utah_flat_7k",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &utah_flat_7k,
        );

        // Utah teapot smooth 7k
        let utah_smooth_7k_data = include_str!("../resources/meshes/utah_teapot_smooth_7k.obj");
        let utah_smooth_7k = format::wavefront::Obj::parse(
            utah_smooth_7k_data.lines().map(String::from),
            "Utah_smooth_7k",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &utah_smooth_7k,
        );

        // Utah teapot smooth 116k
        let utah_smooth_116k_data = include_str!("../resources/meshes/utah_teapot_smooth_116k.obj");
        let utah_smooth_116k = format::wavefront::Obj::parse(
            utah_smooth_116k_data.lines().map(String::from),
            "Utah_smooth_116k",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &utah_smooth_116k,
        );

        // Stanford dragon flat 17k
        let stanford_dragon_flat_17k_data =
            include_str!("../resources/meshes/stanford_dragon_flat_17k.obj");
        let stanford_dragon_flat_17k = format::wavefront::Obj::parse(
            stanford_dragon_flat_17k_data.lines().map(String::from),
            "Stanford_dragon_flat_17k",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &stanford_dragon_flat_17k,
        );

        // Stanford dragon smooth 17k
        let stanford_dragon_smooth_17k_data =
            include_str!("../resources/meshes/stanford_dragon_smooth_17k.obj");
        let stanford_dragon_smooth_17k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_17k_data.lines().map(String::from),
            "Stanford_dragon_smooth_17k",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &stanford_dragon_smooth_17k,
        );

        // Stanford dragon smooth 700k
        let stanford_dragon_smooth_700k_data =
            include_str!("../resources/meshes/stanford_dragon_smooth_700k.obj");
        let stanford_dragon_smooth_700k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_700k_data.lines().map(String::from),
            "Stanford_dragon_smooth_700k",
        );

        append_model_to_vertex_buffer(
            &mut vertex_buffer_data,
            &mut vertex_buffer_offsets,
            &mut vertex_total,
            &mut vertex_counts,
            &stanford_dragon_smooth_700k,
        );

        // (model matrix + normal matrix) * float size, vertex_index_offset + 12 bytes of padding
        let entity_uniform_size = (16 + 12) * 4 + 4 + 12;
        let entity_uniform_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment) as u32
        };

        let entities = {
            [
                // Suzanne flat 967
                Entity {
                    model_matrix: graphic::transform::translate(0.0, 0.0, -5.0)
                        * graphic::transform::scale(2.0, 2.0, 2.0),
                },
                // Suzanne flat 967 messed up normals
                Entity {
                    model_matrix: graphic::transform::translate(-10.0, 0.0, -5.0)
                        * graphic::transform::scale(1.0, 1.0, 1.0),
                },
                // Suzanne smooth 967
                Entity {
                    model_matrix: graphic::transform::translate(5.0, 0.0, -5.0)
                        * graphic::transform::scale(2.0, 2.0, 2.0),
                },
                // Suzanne smooth 967 messed up normals
                Entity {
                    model_matrix: graphic::transform::translate(-5.0, 0.0, -5.0)
                        * graphic::transform::scale(1.0, 1.0, 1.0),
                },
                // Utah teapot flat 7k
                Entity {
                    model_matrix: graphic::transform::translate(0.0, -0.2, -10.0)
                        * graphic::transform::scale(0.5, 0.5, 0.5),
                },
                // Utah teapot smooth 7k
                Entity {
                    model_matrix: graphic::transform::translate(5.0, -0.2, -10.0)
                        * graphic::transform::scale(0.5, 0.5, 0.5),
                },
                // Utah teapot smooth 116k
                Entity {
                    model_matrix: graphic::transform::translate(10.0, -0.2, -10.0)
                        * graphic::transform::scale(0.5, 0.5, 0.5),
                },
                // Stanford dragon flat 17k
                Entity {
                    model_matrix: graphic::transform::translate(0.0, 0.0, -15.0)
                        * graphic::transform::scale(18.0, 18.0, 18.0),
                },
                // Stanford dragon smooth 17k
                Entity {
                    model_matrix: graphic::transform::translate(5.0, 0.0, -15.0)
                        * graphic::transform::scale(18.0, 18.0, 18.0),
                },
                // Stanford dragon smooth 700k
                Entity {
                    model_matrix: graphic::transform::translate(10.0, 0.0, -15.0)
                        * graphic::transform::scale(18.0, 18.0, 18.0),
                },
            ]
            .into_iter()
            .collect::<Vec<Entity>>()
        };

        // Bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind_group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(entity_uniform_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Global Uniform buffer
        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            // uniforms have to be padded to a multiple of 8
            #[allow(clippy::identity_op)] // for clearer explanation
            size: (16 + 16) * 4, // (view matrix, view projection matrix) * float size
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Entity Uniform buffer
        let entity_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity uniform buffer"),
            size: entities.len() as u64 * entity_uniform_alignment as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex_buffer"),
            size: vertex_buffer_data.len() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, &vertex_buffer_data);

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &global_uniform_buffer,
                        offset: 0,
                        size: None, // use whole buffer
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &entity_uniform_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(entity_uniform_size),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &vertex_buffer,
                        offset: 0,
                        size: None,
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
                buffers: &[],
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
            entity_uniform_alignment,
            bind_group,
            global_uniform_buffer,
            entity_uniform_buffer,
            vertex_buffer_offsets,
            vertex_counts,
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
        let mut entity_buffer = vec![0; self.entity_uniform_buffer.size() as usize];
        for (i, entity) in self.entities.iter().enumerate() {
            let normal_matrix = {
                let view_model_matrix = *view_matrix * entity.model_matrix;

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
                .model_matrix
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
                .chain(self.vertex_buffer_offsets[i].to_le_bytes())
                // 3 * 4 padding
                .chain([0u32, 0, 0].iter().flat_map(|val| val.to_le_bytes()))
                .collect::<Vec<u8>>();

            unsafe {
                std::ptr::copy(
                    gpu_entity_bytes.as_ptr(),
                    entity_buffer
                        .as_mut_ptr()
                        .add(self.entity_uniform_alignment as usize * i),
                    gpu_entity_bytes.len(),
                );
            }
        }
        queue.write_buffer(&self.entity_uniform_buffer, 0, &entity_buffer);

        // Serialize to the gpu
        // WGPU works with row major matrices
        let transposed_view_matrix = view_matrix.transpose();
        let transposed_view_projection_matrix = view_projection_matrix.transpose();

        render_pass.set_pipeline(&self.render_pipeline);

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
            .collect::<Vec<u8>>();

        queue.write_buffer(&self.global_uniform_buffer, 0, &global_uniforms);

        // entities
        for i in 0..self.entities.len() {
            render_pass.set_bind_group(
                0,
                &self.bind_group,
                &[self.entity_uniform_alignment * i as u32],
            );
            render_pass.draw(0..self.vertex_counts[i], 0..1);
        }
    }
}
