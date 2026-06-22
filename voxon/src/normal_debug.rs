use graphic::camera::Camera;
use lina::matrix::Matrix;

use std::borrow::Cow;
use wgpu::RenderPipeline;
use wgpu::{
    BindGroupEntry, BufferBinding, BufferUsages, DepthBiasState, DepthStencilState, Face,
    StencilState,
};

#[derive(Debug)]
struct Instance {
    model_matrix: Matrix<f32, 4, 4>,
}

// An Entity represents a Mesh in and its LODs (in the future).
// Each instance defines the transformations applied to each vertex.
#[derive(Debug)]
struct Entity {
    // Our data remains relatively stable, i.e no meshes change and no transform
    // matrices can change for a given entity. So for now, we don't need to maintain
    // the individual buffers, rather the constructed BindGroup will be enough.
    bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    // For different LOD levels, we could turn this into a vector.
    vertex_count: u32,
}

fn construct_entity(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    global_uniform_buffer: &wgpu::Buffer,
    object: &format::wavefront::Obj,
    instances: Vec<Instance>,
) -> Entity {
    let vertex_data = object
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
    let vertex_count = (object.faces().len() * 3) as u32;

    // Instance Storage buffer
    // (model matrix + normal matrix) * float size
    let instance_storage_size = (16 + 12) * 4;
    let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Instance storage buffer"),
        size: instance_storage_size * instances.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vertex_buffer"),
        size: vertex_data.len() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, &vertex_data);

    // Create bind group
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bind_group"),
        layout: bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: global_uniform_buffer,
                    offset: 0,
                    size: None, // use whole buffer
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &instance_buffer,
                    offset: 0,
                    size: None,
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

    Entity {
        bind_group,
        instance_buffer,
        instances,
        vertex_count,
    }
}

#[derive(Debug)]
pub struct NormalDebug {
    // Non-instanced entities
    // Prepared render pipeline and all the necessary info for rendering the scene
    render_pipeline: RenderPipeline,
    global_uniform_buffer: wgpu::Buffer,
    entities: Vec<Entity>,
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

        // Global Uniform buffer
        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            // uniforms have to be padded to a multiple of 8
            #[allow(clippy::identity_op)] // for clearer explanation
            size: (16 + 16) * 4, // (view matrix, view projection matrix) * float size
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
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

        // Construct the entities and their respective bind_groups
        let mut entities = Vec::<Entity>::new();

        // SUZANNE flat 967
        let suzanne_flat_967_data = include_str!("../resources/meshes/suzanne_flat_967.obj");
        let suzanne_flat_967 = format::wavefront::Obj::parse(
            suzanne_flat_967_data.lines().map(String::from),
            "Suzanne_flat_967",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &suzanne_flat_967,
            vec![Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }],
        ));

        // SUZANNE flat 967 messed up normals
        let suzanne_flat_967_messed_up_normals_data =
            include_str!("../resources/meshes/suzanne_flat_967_messed_up_normals.obj");
        let suzanne_flat_967_messed_up_normals = format::wavefront::Obj::parse(
            suzanne_flat_967_messed_up_normals_data
                .lines()
                .map(String::from),
            "Suzanne_flat_967_messed_up_normals",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &suzanne_flat_967_messed_up_normals,
            vec![Instance {
                model_matrix: graphic::transform::translate(-10.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        ));

        // SUZANNE smooth 967
        let suzanne_smooth_967_data = include_str!("../resources/meshes/suzanne_smooth_967.obj");
        let suzanne_smooth_967 = format::wavefront::Obj::parse(
            suzanne_smooth_967_data.lines().map(String::from),
            "Suzanne_smooth_967",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &suzanne_smooth_967,
            vec![Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }],
        ));

        // SUZANNE smooth 967 messed up normals
        let suzanne_smooth_967_messed_up_normals_data =
            include_str!("../resources/meshes/suzanne_smooth_967_messed_up_normals.obj");
        let suzanne_smooth_967_messed_up_normals = format::wavefront::Obj::parse(
            suzanne_smooth_967_messed_up_normals_data
                .lines()
                .map(String::from),
            "Suzanne_smooth_967_messed_up_normals",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &suzanne_smooth_967_messed_up_normals,
            vec![Instance {
                model_matrix: graphic::transform::translate(-5.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        ));

        // Utah teapot flat 7k
        let utah_flat_7k_data = include_str!("../resources/meshes/utah_teapot_flat_7k.obj");
        let utah_flat_7k = format::wavefront::Obj::parse(
            utah_flat_7k_data.lines().map(String::from),
            "Utah_flat_7k",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &utah_flat_7k,
            vec![Instance {
                model_matrix: graphic::transform::translate(0.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        ));

        // Utah teapot smooth 7k
        let utah_smooth_7k_data = include_str!("../resources/meshes/utah_teapot_smooth_7k.obj");
        let utah_smooth_7k = format::wavefront::Obj::parse(
            utah_smooth_7k_data.lines().map(String::from),
            "Utah_smooth_7k",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &utah_smooth_7k,
            vec![Instance {
                model_matrix: graphic::transform::translate(5.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        ));

        // Utah teapot smooth 116k
        let utah_smooth_116k_data = include_str!("../resources/meshes/utah_teapot_smooth_116k.obj");
        let utah_smooth_116k = format::wavefront::Obj::parse(
            utah_smooth_116k_data.lines().map(String::from),
            "Utah_smooth_116k",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &utah_smooth_116k,
            vec![Instance {
                model_matrix: graphic::transform::translate(10.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        ));

        // Stanford dragon flat 17k
        let stanford_dragon_flat_17k_data =
            include_str!("../resources/meshes/stanford_dragon_flat_17k.obj");
        let stanford_dragon_flat_17k = format::wavefront::Obj::parse(
            stanford_dragon_flat_17k_data.lines().map(String::from),
            "Stanford_dragon_flat_17k",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &stanford_dragon_flat_17k,
            vec![Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        ));

        // Stanford dragon smooth 17k
        let stanford_dragon_smooth_17k_data =
            include_str!("../resources/meshes/stanford_dragon_smooth_17k.obj");
        let stanford_dragon_smooth_17k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_17k_data.lines().map(String::from),
            "Stanford_dragon_smooth_17k",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &stanford_dragon_smooth_17k,
            vec![Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        ));

        // Stanford dragon smooth 700k
        let stanford_dragon_smooth_700k_data =
            include_str!("../resources/meshes/stanford_dragon_smooth_700k.obj");
        let stanford_dragon_smooth_700k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_700k_data.lines().map(String::from),
            "Stanford_dragon_smooth_700k",
        );

        entities.push(construct_entity(
            device,
            queue,
            &bind_group_layout,
            &global_uniform_buffer,
            &stanford_dragon_smooth_700k,
            vec![Instance {
                model_matrix: graphic::transform::translate(10.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        ));

        Self {
            render_pipeline,
            global_uniform_buffer,
            entities,
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
        // Serialize to the gpu
        // WGPU works with row major matrices
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
            .collect::<Vec<u8>>();
        queue.write_buffer(&self.global_uniform_buffer, 0, &global_uniforms);

        // Update entity storage buffers
        for entity in &self.entities {
            let mut instance_buffer =
                vec![0; entity.instances.len() * entity.instance_buffer.size() as usize];

            for (i, instance) in entity.instances.iter().enumerate() {
                let normal_matrix = {
                    let view_model_matrix = *view_matrix * instance.model_matrix;

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

                let gpu_instance_bytes = instance
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
                    .collect::<Vec<u8>>();

                unsafe {
                    std::ptr::copy(
                        gpu_instance_bytes.as_ptr(),
                        instance_buffer
                            .as_mut_ptr()
                            .add(entity.instance_buffer.size() as usize * i),
                        gpu_instance_bytes.len(),
                    );
                }
            }
            queue.write_buffer(&entity.instance_buffer, 0, &instance_buffer);
        }

        render_pass.set_pipeline(&self.render_pipeline);

        // Emit the draw calls
        // entities
        for entity in &self.entities {
            render_pass.set_bind_group(0, Some(&entity.bind_group), &[]);
            render_pass.draw(0..entity.vertex_count, 0..entity.instances.len() as u32);
        }
    }
}
