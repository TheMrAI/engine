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

    let texture = upload_texture(device, queue, dimensions, bytes, true);

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

    let texture = upload_texture(device, queue, dimensions, bytes, true);

    texture.create_view(&wgpu::wgt::TextureViewDescriptor::default())
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
    texture_buffers: &[u8],
    generate_mips: bool,
) -> wgpu::Texture {
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
    queue.write_texture(
        texture.as_image_copy(),
        texture_buffers,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(dimensions.width * 4),
            rows_per_image: None,
        },
        single_texture_extent,
    );

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

    let views = (0..mip_count)
        .map(|mip_level| {
            texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("mip_level"),
                format: None,
                dimension: None,
                usage: Some(
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
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
