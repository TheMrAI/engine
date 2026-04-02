pub fn load_texture_plane(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let image_data = include_bytes!("../resources/textures/texture_01.png");
    let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame_info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..frame_info.buffer_size()];

    let dimensions = wgpu::Extent3d {
        width: frame_info.width,
        height: frame_info.height,
        depth_or_array_layers: 1,
    };

    let texture = upload_texture(device, queue, dimensions, &[bytes], true);

    texture.create_view(&wgpu::wgt::TextureViewDescriptor::default())
}

pub fn load_texture_cube(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let image_data = include_bytes!("../resources/textures/cube_atlas.png");
    let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame_info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..frame_info.buffer_size()];

    let dimensions = wgpu::Extent3d {
        width: frame_info.width,
        height: frame_info.height,
        depth_or_array_layers: 1,
    };

    let texture = upload_texture(device, queue, dimensions, &[bytes], true);

    texture.create_view(&wgpu::wgt::TextureViewDescriptor::default())
}

pub fn load_cubemap_textures(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    // This is nasty, as the textures are compiled into the binary, but for now it is okay.
    // px
    let image_data = include_bytes!("../resources/textures/skybox/sky_cube_px.png");
    let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame_info = reader.next_frame(&mut buf).unwrap();
    let px_bytes = &buf[..frame_info.buffer_size()];
    // nx
    let image_data = include_bytes!("../resources/textures/skybox/sky_cube_nx.png");
    let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame_info = reader.next_frame(&mut buf).unwrap();
    let nx_bytes = &buf[..frame_info.buffer_size()];
    // py
    let image_data = include_bytes!("../resources/textures/skybox/sky_cube_py.png");
    let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame_info = reader.next_frame(&mut buf).unwrap();
    let py_bytes = &buf[..frame_info.buffer_size()];
    // ny
    let image_data = include_bytes!("../resources/textures/skybox/sky_cube_ny.png");
    let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame_info = reader.next_frame(&mut buf).unwrap();
    let ny_bytes = &buf[..frame_info.buffer_size()];
    // pz
    let image_data = include_bytes!("../resources/textures/skybox/sky_cube_pz.png");
    let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame_info = reader.next_frame(&mut buf).unwrap();
    let pz_bytes = &buf[..frame_info.buffer_size()];
    // nz
    let image_data = include_bytes!("../resources/textures/skybox/sky_cube_nz.png");
    let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
    let mut reader = png_decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let frame_info = reader.next_frame(&mut buf).unwrap();
    let nz_bytes = &buf[..frame_info.buffer_size()];

    // The order of the buffers matters.
    // See: https://gpuweb.github.io/gpuweb/#dom-gputextureviewdimension-cube
    let cubemap_buffers = &[px_bytes, nx_bytes, py_bytes, ny_bytes, pz_bytes, nz_bytes];

    // This part is confusing.
    // A dimension struct defines that we are working with a texture that has 3 dimensions.
    // Do not be confused though. It seems that for a cubemap **..or_array_layers** part is
    // the important one. In case, the third value does not represent **depth**, then we
    // should not think of a texture as 3 dimensional. It merely has multiple layers.
    let dimensions = wgpu::Extent3d {
        width: frame_info.width,
        height: frame_info.height,
        depth_or_array_layers: 6,
    };

    let texture = upload_texture(device, queue, dimensions, cubemap_buffers, true);

    // A texture view is created, which says the underlying texture satisfies
    // the requirements of being a cube(map).
    texture.create_view(&wgpu::wgt::TextureViewDescriptor {
        label: Some("cube_view"),
        format: None,
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None, // include all mipmap levels
        base_array_layer: 0,
        array_layer_count: Some(6),
    })
}

// Upload a 2D texture to the GPU and generate mipmaps if requested.
//
// It does not support textures using multiple layers, like cubemaps.
// The texture buffer has to have the RGBA values using the sRGB color
// space.
pub fn upload_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    dimensions: wgpu::Extent3d,
    texture_buffers: &[&[u8]],
    generate_mips: bool,
) -> wgpu::Texture {
    debug_assert!(dimensions.depth_or_array_layers > 0);
    debug_assert!(texture_buffers.len() == dimensions.depth_or_array_layers as usize);

    let mip_level_count = {
        if generate_mips {
            std::cmp::max(dimensions.width.ilog2(), dimensions.height.ilog2())
        } else {
            1
        }
    };

    // Create the top level texture.
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hand_texture"),
        size: dimensions,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        mip_level_count,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        sample_count: 1,
        view_formats: &[],
        dimension: wgpu::TextureDimension::D2,
    });

    let single_texture_extent = wgpu::Extent3d {
        depth_or_array_layers: 1,
        ..dimensions
    };

    if dimensions.depth_or_array_layers == 1 {
        queue.write_texture(
            texture.as_image_copy(),
            texture_buffers[0],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(dimensions.width * 4),
                rows_per_image: None,
            },
            single_texture_extent,
        );
    } else {
        for layer in 0..dimensions.depth_or_array_layers {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                texture_buffers[layer as usize],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(dimensions.width * 4),
                    rows_per_image: None,
                },
                single_texture_extent,
            );
        }
    }

    if generate_mips {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mimpap_encoder"),
        });
        generate_mipmap(device, &mut encoder, &texture);
        queue.submit(Some(encoder.finish()));
    }

    texture
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
    let mipmap_shader = device.create_shader_module(wgpu::include_wgsl!("mipmap.wgsl"));

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mipmap_pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &mipmap_shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &mipmap_shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(texture.format().into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
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

    let layer_views = (0..texture.depth_or_array_layers())
        .map(|layer| {
            let array_layer_count = if texture.depth_or_array_layers() == 1 {
                None
            } else {
                Some(1)
            };

            // views
            (0..mip_count)
                .map(|mip_level| {
                    texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("mip_level"),
                        format: None,
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        usage: Some(
                            wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::RENDER_ATTACHMENT,
                        ),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: mip_level,
                        mip_level_count: Some(1),
                        base_array_layer: layer,
                        array_layer_count,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    for mip_level in 1..mip_count as usize {
        for views in &layer_views {
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
}
