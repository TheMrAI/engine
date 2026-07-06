use crate::MeshBuffer;
use crate::pipeline::Pipeline;
use lina::matrix::resize;
use lina::matrix::{Matrix, Resize};
use scene::MeshNode;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use std::borrow::Cow;
use wgpu::RenderPipeline;
use wgpu::{BindGroupLayoutEntry, DepthBiasState, DepthStencilState, Face, StencilState};

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

#[derive(Debug)]
pub struct NormalDebugWireframe {
    // Non-instanced entities
    // Prepared render pipeline and all the necessary info for rendering the scene
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: RenderPipeline,
    scheduled_entities: HashMap<u32, EntityInstanceGroups>,
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

        Self {
            bind_group_layout,
            render_pipeline,
            scheduled_entities: Default::default(),
        }
    }
}

impl Pipeline for NormalDebugWireframe {
    fn schedule_render(&mut self, mesh_node: Rc<RefCell<MeshNode>>) {
        self.scheduled_entities
            .entry(mesh_node.borrow().mesh_id)
            .or_default()
            .0
            .push(mesh_node.clone());
    }

    #[allow(clippy::too_many_arguments)]
    fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        mesh_cache: &std::collections::HashMap<u32, MeshBuffer>,
        _texture_cache: &std::collections::HashMap<u32, wgpu::Texture>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_matrix: &Matrix<f32, 4, 4>,
        _view_projection_matrix: &Matrix<f32, 4, 4>,
        global_uniform_buffer: &wgpu::Buffer,
    ) {
        for (mesh_id, instances) in &mut self.scheduled_entities {
            let mesh_buffer = mesh_cache.get(mesh_id).unwrap();

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
