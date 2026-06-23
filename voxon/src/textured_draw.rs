use graphic::camera::Camera;
use lina::matrix::Matrix;

use std::borrow::Cow;
use wgpu::RenderPipeline;
use wgpu::{BufferUsages, DepthBiasState, DepthStencilState, Face, StencilState};

#[derive(Debug)]
pub struct Instance {
    pub model_matrix: Matrix<f32, 4, 4>,
}

#[derive(Debug)]
pub struct TextureInstance {
    pub texture_scale: f32,
}

#[derive(Debug)]
struct Entity {
    bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    texture_instance_buffer: wgpu::Buffer,
    texture_instances: Vec<TextureInstance>,
    vertex_count: u32,
}

#[derive(Debug)]
pub struct Textured {
    // Prepared render pipeline and all the necessary info for rendering the scene
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: RenderPipeline,
    global_uniform_buffer: wgpu::Buffer,
    entities: Vec<Entity>,
    sampler: wgpu::Sampler,
}

impl Textured {
    pub fn new(device: &wgpu::Device, color_target: wgpu::ColorTargetState) -> Self {
        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("textured_draw"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("textured_draw.wgsl"))),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
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

        let entities = Vec::<Entity>::new();

        Self {
            bind_group_layout,
            render_pipeline,
            global_uniform_buffer,
            entities,
            sampler,
        }
    }

    pub fn add_entity_instances(
        &mut self,
        device: &wgpu::Device,
        vertex_buffer: &wgpu::Buffer,
        vertex_count: u32,
        instances: Vec<Instance>,
        texture_instances: Vec<TextureInstance>,
        texture: wgpu::Texture,
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

        let texture_instance_storage_size = 4 * 4;
        let texture_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance storage buffer"),
            size: texture_instance_storage_size * instances.len() as u64,
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
                        buffer: &self.global_uniform_buffer,
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &texture_instance_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::wgt::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        self.entities.push(Entity {
            bind_group,
            instance_buffer,
            instances,
            vertex_count,
            texture_instance_buffer,
            texture_instances,
        })
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
        queue.write_buffer(&self.global_uniform_buffer, 0, &global_uniforms);

        // Update entity storage buffers
        for entity in &self.entities {
            let instance_size = entity.instance_buffer.size() as usize / entity.instances.len();
            let mut instance_buffer = vec![0; entity.instance_buffer.size() as usize];

            for (i, instance) in entity.instances.iter().enumerate() {
                let gpu_instance_bytes = instance
                    .model_matrix
                    .transpose()
                    .as_slices()
                    .iter()
                    .flatten()
                    .flat_map(|entry| entry.to_le_bytes())
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

            let texture_instance_size =
                entity.texture_instance_buffer.size() as usize / entity.texture_instances.len();
            let mut texture_instance_buffer =
                vec![0; entity.texture_instance_buffer.size() as usize];
            for (i, instance) in entity.texture_instances.iter().enumerate() {
                let gpu_instance_bytes = [instance.texture_scale]
                    .iter()
                    .flat_map(|entry| entry.to_le_bytes())
                    .collect::<Vec<u8>>();

                unsafe {
                    std::ptr::copy(
                        gpu_instance_bytes.as_ptr(),
                        texture_instance_buffer
                            .as_mut_ptr()
                            .add(texture_instance_size * i),
                        gpu_instance_bytes.len(),
                    );
                }
            }
            queue.write_buffer(&entity.texture_instance_buffer, 0, &texture_instance_buffer);
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
