//! Scene with a single cubemapped cube in it.
//!
//! Can only be rendered as part of another scene which handles the camera
//! and it's view transformations.
use lina::matrix::Matrix;
use wgpu::{BindingResource, VertexAttribute, VertexBufferLayout};

use crate::texture::load_cubemap_textures;
use mesh::generate_cube;

// TODO Code is dead. Has been for a while, still
// keeping it until it is clear it won't be used.

#[allow(dead_code)]
pub struct CubeMap {
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    index_count: usize,
}

impl CubeMap {
    #[allow(dead_code)]
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_target: wgpu::ColorTargetState,
    ) -> Self {
        let cube_mesh = generate_cube();
        let cube_vertex_data = cube_mesh
            .vertices()
            .iter()
            .flat_map(|entry| {
                entry
                    .position()
                    .as_slice()
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
            })
            .collect::<Vec<u8>>();

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_vertex_buffer"),
            size: cube_vertex_data.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, &cube_vertex_data);

        let cube_index_data = cube_mesh
            .indices()
            .iter()
            .flat_map(|index| index.to_le_bytes())
            .collect::<Vec<_>>();
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_index_buffer"),
            size: cube_index_data.len() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&index_buffer, 0, &cube_index_data);

        let cube_map_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cube_map_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "cube_map.wgsl"
            ))),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube_map"),
            layout: None, // auto pipeline layout,
            vertex: wgpu::VertexState {
                module: &cube_map_shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: 4 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
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

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_map_uniforms"),
            size: 16 * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
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

        let texture_view = load_cubemap_textures(device, queue);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cube_map_bind_group"),
            layout: &render_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&texture_view),
                },
            ],
        });

        Self {
            render_pipeline,
            uniform_buffer,
            bind_group,
            vertex_buffer,
            index_buffer,
            index_count: cube_mesh.indices().len(),
            index_format: wgpu::IndexFormat::Uint32,
        }
    }

    #[allow(dead_code)]
    pub fn render(
        &self,
        render_pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        view_projection_matrix: Matrix<f32, 4, 4>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);

        let uniforms = view_projection_matrix
            .as_slices()
            .iter()
            .flatten()
            .flat_map(|entry| entry.to_le_bytes())
            .collect::<Vec<u8>>();

        queue.write_buffer(&self.uniform_buffer, 0, &uniforms);

        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_index_buffer(self.index_buffer.slice(..), self.index_format);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
