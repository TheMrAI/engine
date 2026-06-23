use graphic::camera::Camera;
use lina::matrix::Matrix;

use std::borrow::Cow;
use wgpu::RenderPipeline;
use wgpu::{BindGroupLayoutEntry, DepthBiasState, DepthStencilState, Face, StencilState};

#[derive(Debug)]
pub struct Instance {
    pub model_matrix: Matrix<f32, 4, 4>,
}

#[derive(Debug)]
struct Entity {
    // Our data remains relatively stable, i.e no meshes change and no transform
    // matrices can change for a given entity. So for now, we don't need to maintain
    // the individual buffers, rather the constructed BindGroup will be enough.
    bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    vertex_count: u32,
}

#[derive(Debug)]
pub struct NormalDebugWireframe {
    // Non-instanced entities
    // Prepared render pipeline and all the necessary info for rendering the scene
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: RenderPipeline,
    entities: Vec<Entity>,
}

impl NormalDebugWireframe {
    pub fn new(device: &wgpu::Device, color_target: wgpu::ColorTargetState) -> Self {
        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "normal_debug_wireframe.wgsl"
            ))),
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
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
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

        let entities = Vec::<Entity>::new();

        Self {
            bind_group_layout,
            render_pipeline,
            entities,
        }
    }

    pub fn add_entity_instances(
        &mut self,
        device: &wgpu::Device,
        global_uniform_buffer: &wgpu::Buffer,
        vertex_buffer: &wgpu::Buffer,
        vertex_count: u32,
        instances: Vec<Instance>,
    ) {
        // Instance Storage buffer
        // (model matrix + normal matrix) * float size
        let instance_storage_size = (16 + 12) * 4;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance storage buffer"),
            size: instance_storage_size * instances.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
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
                        buffer: vertex_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        self.entities.push(Entity {
            bind_group,
            instance_buffer,
            instances,
            vertex_count,
        })
    }

    pub fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        _camera: &Camera,
        view_matrix: &Matrix<f32, 4, 4>,
        _view_projection_matrix: &Matrix<f32, 4, 4>,
    ) {
        // Update entity storage buffers
        for entity in &self.entities {
            let instance_size = entity.instance_buffer.size() as usize / entity.instances.len();
            let mut instance_buffer = vec![0; entity.instance_buffer.size() as usize];

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
                        instance_buffer.as_mut_ptr().add(instance_size * i),
                        gpu_instance_bytes.len(),
                    );
                }
            }
            queue.write_buffer(&entity.instance_buffer, 0, &instance_buffer);
        }

        render_pass.set_pipeline(&self.render_pipeline);

        // entities
        for entity in &self.entities {
            render_pass.set_bind_group(0, Some(&entity.bind_group), &[]);
            render_pass.draw(0..entity.vertex_count, 0..entity.instances.len() as u32);
        }
    }
}
