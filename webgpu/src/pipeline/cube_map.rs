use crate::MeshBuffer;
use crate::pipeline::Pipeline;
use lina::matrix::resize;
use lina::matrix::{Matrix, Resize};
use scene::MeshNode;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Instance Storage buffer
// (model matrix + normal matrix) * float size
static INSTANCE_STORAGE_SIZE: u64 = (16 + 12) * 4;
type EntityInstanceGroups = (Vec<Rc<RefCell<MeshNode>>>, Option<InstanceCache>);

#[derive(Debug)]
struct InstanceCache {
    instance_buffer_data: Vec<u8>,
    instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Id {
    mesh_id: u32,
    texture_id: u32,
}

#[derive(Debug)]
pub struct CubeMap {
    // Prepared render pipeline and all the necessary info for rendering the scene
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    scheduled_entities: HashMap<Id, EntityInstanceGroups>,
}

impl CubeMap {
    pub fn new(device: &wgpu::Device, color_target: wgpu::ColorTargetState) -> Self {
        let cube_map_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cube_map_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "cube_map.wgsl"
            ))),
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
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        // pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube_map"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &cube_map_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &cube_map_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(color_target)],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_compare: wgpu::CompareFunction::Less,
                depth_write_enabled: true,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("environment_map_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        Self {
            bind_group_layout,
            render_pipeline,
            sampler,
            scheduled_entities: Default::default(),
        }
    }
}

impl Pipeline for CubeMap {
    fn schedule_render(&mut self, mesh_node: Rc<RefCell<MeshNode>>) {
        let id = Id {
            mesh_id: mesh_node.borrow().mesh_id,
            texture_id: mesh_node.borrow().texture_id.unwrap(),
        };
        self.scheduled_entities
            .entry(id)
            .or_default()
            .0
            .push(mesh_node.clone());
    }

    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        mesh_cache: &std::collections::HashMap<u32, MeshBuffer>,
        texture_cache: &std::collections::HashMap<u32, wgpu::Texture>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_matrix: &Matrix<f32, 4, 4>,
        _view_projection_matrix: &Matrix<f32, 4, 4>,
        global_uniform_buffer: &wgpu::Buffer,
    ) {
        for (id, instances) in &mut self.scheduled_entities {
            let mesh_buffer = mesh_cache.get(&id.mesh_id).unwrap();
            let texture = texture_cache.get(&id.texture_id).unwrap();

            let instance_cache = {
                let rebuild = match &instances.1 {
                    Some(cache) => {
                        cache.instance_buffer_data.len()
                            < INSTANCE_STORAGE_SIZE as usize * instances.0.len()
                    }
                    None => true,
                };
                if rebuild {
                    let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Instance storage buffer"),
                        size: INSTANCE_STORAGE_SIZE * instances.0.len() as u64,
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
                                    buffer: &mesh_buffer.vertex_buffer,
                                    offset: 0,
                                    size: None,
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: wgpu::BindingResource::TextureView(&texture.create_view(
                                    &wgpu::wgt::TextureViewDescriptor {
                                        label: Some("cube_view"),
                                        format: None,
                                        dimension: Some(wgpu::TextureViewDimension::Cube),
                                        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                                        aspect: wgpu::TextureAspect::All,
                                        base_mip_level: 0,
                                        mip_level_count: None, // include all mipmap levels
                                        base_array_layer: 0,
                                        array_layer_count: Some(6),
                                    },
                                )),
                            },
                        ],
                    });

                    let instance_buffer_data = vec![0; instance_buffer.size() as usize];

                    instances.1 = Some(InstanceCache {
                        instance_buffer_data,
                        instance_buffer,
                        bind_group,
                    });
                }
                instances.1.as_mut().unwrap()
            };

            for (i, mesh_node) in instances.0.iter().enumerate() {
                let normal_matrix = {
                    let view_model_matrix = *view_matrix * mesh_node.borrow().model_matrix;

                    let matrix = resize!(view_model_matrix, 3, 3);
                    matrix.adjoint()
                };

                let padded_flattened_normal_matrix = resize!(normal_matrix, 4, 3);

                let gpu_instance_bytes = mesh_node
                    .borrow()
                    .model_matrix
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
                        instance_cache
                            .instance_buffer_data
                            .as_mut_ptr()
                            .add(INSTANCE_STORAGE_SIZE as usize * i),
                        gpu_instance_bytes.len(),
                    );
                }
            }
            queue.write_buffer(
                &instance_cache.instance_buffer,
                0,
                &instance_cache.instance_buffer_data,
            );

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, Some(&instance_cache.bind_group), &[]);
            render_pass.draw(0..mesh_buffer.vertex_count, 0..instances.0.len() as u32);

            // clear instance ids
            instances.0.clear();
        }
    }
}
