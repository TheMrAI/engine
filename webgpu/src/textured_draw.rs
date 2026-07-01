use lina::matrix::resize;
use lina::matrix::{Matrix, Resize};

use std::borrow::Cow;
use wgpu::RenderPipeline;
use wgpu::{DepthBiasState, DepthStencilState, Face, StencilState};

#[derive(Debug)]
pub struct Textured {
    // Prepared render pipeline and all the necessary info for rendering the scene
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: RenderPipeline,
    sampler: wgpu::Sampler,
}

impl Textured {
    pub fn new(device: &wgpu::Device, color_target: wgpu::ColorTargetState) -> Self {
        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("textured_draw"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("textured_draw.wgsl"))),
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

        Self {
            bind_group_layout,
            render_pipeline,
            sampler,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_matrix: &Matrix<f32, 4, 4>,
        _view_projection_matrix: &Matrix<f32, 4, 4>,
        global_uniform_buffer: &wgpu::Buffer,
        instances: &[(Matrix<f32, 4, 4>, f32)],
        vertex_buffer: &wgpu::Buffer,
        vertex_count: u32,
        texture: &wgpu::Texture,
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

        let mut instance_buffer_data = vec![0; instance_buffer.size() as usize];

        for (i, (model_matrix, _)) in instances.iter().enumerate() {
            let normal_matrix = {
                let view_model_matrix = *view_matrix * *model_matrix;

                let matrix = resize!(view_model_matrix, 3, 3);
                matrix.adjoint()
            };

            let padded_flattened_normal_matrix = resize!(normal_matrix, 4, 3);

            let gpu_instance_bytes = model_matrix
                .transpose()
                .as_slices()
                .iter()
                .flatten()
                .flat_map(|entry| entry.to_le_bytes())
                .chain(
                    padded_flattened_normal_matrix
                        .as_slices()
                        .iter()
                        .flatten()
                        .flat_map(|entry| entry.to_le_bytes()),
                )
                .collect::<Vec<u8>>();

            unsafe {
                std::ptr::copy(
                    gpu_instance_bytes.as_ptr(),
                    instance_buffer_data
                        .as_mut_ptr()
                        .add(instance_storage_size as usize * i),
                    gpu_instance_bytes.len(),
                );
            }
        }
        queue.write_buffer(&instance_buffer, 0, &instance_buffer_data);

        let mut texture_instance_buffer_data = vec![0; texture_instance_buffer.size() as usize];

        for (i, (_, texture_scale)) in instances.iter().enumerate() {
            let gpu_instance_bytes = [texture_scale]
                .iter()
                .flat_map(|entry| entry.to_le_bytes())
                .collect::<Vec<u8>>();

            unsafe {
                std::ptr::copy(
                    gpu_instance_bytes.as_ptr(),
                    texture_instance_buffer_data
                        .as_mut_ptr()
                        .add(texture_instance_storage_size as usize * i),
                    gpu_instance_bytes.len(),
                );
            }
        }
        queue.write_buffer(&texture_instance_buffer, 0, &texture_instance_buffer_data);

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, Some(&bind_group), &[]);
        render_pass.draw(0..vertex_count, 0..instances.len() as u32);
    }
}
