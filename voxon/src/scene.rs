use std::{
    borrow::Cow,
    f32::consts::PI,
    io::{self},
    time::Duration,
};

use graphic::{camera::Camera, identity_matrix};
use lina::{m, matrix::Matrix, v};

use quaternion::Quaternion;
use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferBinding, BufferUsages, DepthBiasState, DepthStencilState,
    Device, Extent3d, Face, FragmentState, MultisampleState, Operations, PrimitiveState, Queue,
    RenderPassDepthStencilAttachment, RenderPipeline, RenderPipelineDescriptor, StencilState,
    Surface, TexelCopyBufferLayout, TextureDescriptor, TextureUsages, VertexAttribute,
    VertexBufferLayout, VertexState, include_wgsl, util::align_to, wgt::CommandEncoderDescriptor,
};
use winit::dpi::PhysicalSize;

use crate::mesh::{generate_cube, generate_plane};

pub struct Entity {
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

//
// A Scene should be a structure which manages the lifetimes
// of any mesh, texture, sound, shader that is used in the scene.
// It can handle the hierarchical scene elements, their transformations etc.
//
// This is not the desired Scene as it has no such structures.
// It also contains logic specifying how it should be transformed into
// commands for the GPU. That should be the domain of a completely different class,
// but for the time being it has been moved here as well.
// Mostly to keep things simple.
pub struct Scene {
    // Delta_t associated with the cube rotation
    cube_delta_t: std::time::Duration,
    // Prepared render pipeline and all the necessary info for rendering the scene
    render_pipeline: RenderPipeline,
    entities: Vec<Entity>,
    global_uniforms: (Buffer, BindGroup),
    entity_uniforms: (Buffer, BindGroup),
    texture_uniforms: BindGroup,
}

impl Scene {
    pub fn new(adapter: &Adapter, surface: &Surface, device: &Device, queue: &Queue) -> Self {
        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        // CUBE
        let cube_mesh = generate_cube();
        let cube_vertex_data = cube_mesh
            .vertices()
            .iter()
            .flat_map(|entry| {
                entry
                    .position()
                    .as_slice()
                    .iter()
                    .chain(entry.normal().as_slice().iter().chain([&0.0]))
                    .chain(entry.uv().as_slice().iter())
                    .flat_map(|value| value.to_le_bytes())
            })
            .collect::<Vec<u8>>();

        let cube_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_vertex_buffer"),
            size: cube_vertex_data.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&cube_vertex_buffer, 0, &cube_vertex_data);

        let cube_index_data = cube_mesh
            .indices()
            .iter()
            .flat_map(|index| index.to_le_bytes())
            .collect::<Vec<_>>();
        let cube_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_index_buffer"),
            size: cube_index_data.len() as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&cube_index_buffer, 0, &cube_index_data);

        // PLANE
        let plane_mesh = generate_plane();
        let plane_vertex_data = plane_mesh
            .vertices()
            .iter()
            .flat_map(|entry| {
                entry
                    .position()
                    .as_slice()
                    .iter()
                    .chain(entry.normal().as_slice().iter().chain([&0.0]))
                    .chain(entry.uv().as_slice().iter())
                    .flat_map(|value| value.to_le_bytes())
            })
            .collect::<Vec<u8>>();

        let plane_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("plane_vertex_buffer"),
            size: plane_vertex_data.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&plane_vertex_buffer, 0, &plane_vertex_data);

        let plane_index_data = plane_mesh
            .indices()
            .iter()
            .flat_map(|index| index.to_le_bytes())
            .collect::<Vec<_>>();
        let plane_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("plane_index_buffer"),
            size: plane_index_data.len() as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&plane_index_buffer, 0, &plane_index_data);

        let entity_uniform_size = (16 + 16) * 4;
        let entity_uniform_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(entity_uniform_size, alignment)
        };

        let entities = {
            [
                Entity {
                    vertex_buffer: cube_vertex_buffer,
                    index_buffer: cube_index_buffer,
                    index_format: wgpu::IndexFormat::Uint32,
                    index_count: cube_mesh.indices().len(),
                    world_matrix: identity_matrix(),
                    normal_matrix: Matrix::<f32, 3, 3>::from_value(0.0),
                    uniform_offset: 0,
                },
                Entity {
                    vertex_buffer: plane_vertex_buffer,
                    index_buffer: plane_index_buffer,
                    index_format: wgpu::IndexFormat::Uint32,
                    index_count: plane_mesh.indices().len(),
                    world_matrix: graphic::transform::translate(0.0, -1.0, 0.0)
                        * graphic::transform::scale(50.0, 1.0, 50.0),
                    normal_matrix: m![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],],
                    uniform_offset: entity_uniform_alignment as u32,
                },
            ]
            .into_iter()
            .collect::<Vec<Entity>>()
        };

        let image_data = include_bytes!("texture_01.png");
        let png_decoder = png::Decoder::new(io::Cursor::new(image_data));
        let mut reader = png_decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let frame_info = reader.next_frame(&mut buf).unwrap();
        let bytes = &buf[..frame_info.buffer_size()];

        let texture_extent = Extent3d {
            width: frame_info.width,
            height: frame_info.height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("hand_texture"),
            size: texture_extent,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            mip_level_count: std::cmp::max(frame_info.width.ilog2(), frame_info.height.ilog2()),
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            sample_count: 1,
            view_formats: &[],
            dimension: wgpu::TextureDimension::D2,
        });

        let texture_view = texture.create_view(&wgpu::wgt::TextureViewDescriptor::default());
        queue.write_texture(
            texture.as_image_copy(),
            bytes,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(frame_info.width * 4),
                rows_per_image: None,
            },
            texture_extent,
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("mimpap_encoder"),
        });
        generate_mipmap(device, &mut encoder, &texture);
        queue.submit(Some(encoder.finish()));

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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_sampler_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
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
        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });
        let texture_uniforms = texture_bind_group;

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
                size: (16 + 3) * 4 + 4, // (view projection matrix + view position) * float size + padding
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
                        min_binding_size: wgpu::BufferSize::new(entity_uniform_size), // (world matrix + normal matrix) * float size, no padding needed
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
                &texture_bind_group_layout,
            ],
            immediate_size: 0,
        });

        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline_descriptor"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: (4 + 3 + 1 + 2) * 4, // (4 floats for position + 3 floats for normal + 1 padding + 2 UV) * f32 byte count
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
                        // uv
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 32,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(swapchain_format.into())],
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
            cube_delta_t: Duration::default(),
            render_pipeline,
            entities,
            global_uniforms,
            entity_uniforms,
            texture_uniforms,
        }
    }

    pub fn simulate(&mut self, delta_t: Duration) {
        // World simulation.
        // It will not be part of the render pipeline later on.
        // Only temporarily for now.

        // For now the cube transformations are hacked in here.
        let cube_full_rotation_time = std::time::Duration::from_secs(10);
        self.cube_delta_t = self.cube_delta_t.saturating_add(delta_t);
        if self.cube_delta_t > cube_full_rotation_time {
            self.cube_delta_t = self.cube_delta_t.saturating_sub(cube_full_rotation_time);
        }

        // for quick rotation checks
        #[allow(unused_variables)]
        let rotate_y: Matrix<f32, 4, 4> = Quaternion::<f32>::new_unit(
            2.0 * PI
                * (self.cube_delta_t.as_millis() as f32
                    / cube_full_rotation_time.as_millis() as f32),
            v![0.0, 1.0, 0.0],
        )
        .into();

        let cube_world_matrix = graphic::identity_matrix();

        let cube_normal_matrix = {
            let mut matrix = Matrix::<f32, 3, 3>::new();
            // may be padded incorrectly!!! check
            matrix[(0, 0)] = cube_world_matrix[(0, 0)];
            matrix[(0, 1)] = cube_world_matrix[(0, 1)];
            matrix[(0, 2)] = cube_world_matrix[(0, 2)];

            matrix[(1, 0)] = cube_world_matrix[(1, 0)];
            matrix[(1, 1)] = cube_world_matrix[(1, 1)];
            matrix[(1, 2)] = cube_world_matrix[(1, 2)];

            matrix[(2, 0)] = cube_world_matrix[(2, 0)];
            matrix[(2, 1)] = cube_world_matrix[(2, 1)];
            matrix[(2, 2)] = cube_world_matrix[(2, 2)];

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

        self.entities[0].world_matrix = cube_world_matrix;
        self.entities[0].normal_matrix = cube_normal_matrix;
    }

    pub fn render(
        &self,
        inner_size: &PhysicalSize<u32>,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
        camera: &Camera,
    ) {
        // Create render texture
        let frame = surface
            .get_current_texture()
            .expect("failed to acquire next swap-chain texture");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("depth texture"),
            size: frame.texture.size(),
            mip_level_count: 1, // no extra mips, has to be 1
            sample_count: 1,    // no multisampling, so 1
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[], // no special view format needed
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        for entity in &self.entities {
            let padded_flattened_normal_matrix = [
                entity.normal_matrix[(0, 0)],
                entity.normal_matrix[(0, 1)],
                entity.normal_matrix[(0, 2)],
                0.0,
                entity.normal_matrix[(1, 0)],
                entity.normal_matrix[(1, 1)],
                entity.normal_matrix[(1, 2)],
                0.0,
                entity.normal_matrix[(2, 0)],
                entity.normal_matrix[(2, 1)],
                entity.normal_matrix[(2, 2)],
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

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);

            // the camera matrix
            let look_at = camera.as_transform_matrix();
            // view matrix
            let view_matrix = look_at;

            let aspect_ratio = inner_size.width as f32 / inner_size.height as f32;
            let projection_matrix = graphic::transform::perspective_proj_sym_h_fov(
                PI / 2.0,
                aspect_ratio,
                -1.0,
                -20000.0,
            );

            let view_projection_matrix = projection_matrix * view_matrix;

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

            queue.write_buffer(&self.global_uniforms.0, 0, &global_uniforms);
            render_pass.set_bind_group(0, &self.global_uniforms.1, &[]);
            render_pass.set_bind_group(2, &self.texture_uniforms, &[]);

            // entities
            for entity in &self.entities {
                render_pass.set_bind_group(1, &self.entity_uniforms.1, &[entity.uniform_offset]);
                render_pass.set_index_buffer(entity.index_buffer.slice(..), entity.index_format);
                render_pass.set_vertex_buffer(0, entity.vertex_buffer.slice(..));
                render_pass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

// Generate mipmaps on the GPU.
//
// The base texture must be uploaded before calling this function.
// The number of mip-maps will always be 'max(log2(base_texture_width), log2(base_texture_height))',
// ensure that the base texture was created such, that it expects exactly the above mipmap level count.
pub fn generate_mipmap(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
) {
    let mip_count = std::cmp::max(texture.width().ilog2(), texture.height().ilog2());
    let mipmap_shader = device.create_shader_module(include_wgsl!("mipmap.wgsl"));

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("mipmap_pipeline"),
        layout: None,
        vertex: VertexState {
            module: &mipmap_shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &mipmap_shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(texture.format().into())],
        }),
        primitive: PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let bind_group_layout = pipeline.get_bind_group_layout(0);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("mipmapper"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let views = (0..mip_count)
        .map(|mip_level| {
            texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("mip_level"),
                format: None,
                dimension: None,
                usage: Some(TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: None,
            })
        })
        .collect::<Vec<_>>();

    for mip_level in 1..mip_count as usize {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mipmap_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&views[mip_level - 1]),
                },
            ],
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("mipmap_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &views[mip_level],
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
