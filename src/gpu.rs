use std::{borrow::Cow, f32::consts::PI, sync::Arc};

use wgpu::{
    Adapter, BindGroup, BindGroupEntry, Buffer, BufferBinding, BufferUsages, DepthBiasState,
    DepthStencilState, Device, Face, Operations, Queue, RenderPassDepthStencilAttachment,
    RenderPipeline, StencilState, Surface, TextureDescriptor, TextureUsages, VertexAttribute,
    VertexBufferLayout,
};
use winit::{dpi::PhysicalSize, window::Window};

// Notice that all transformation matrices are transposed compared
// to how they would appear in an algebra book.
#[rustfmt::skip]
pub fn translate(translate_x: f32, translate_y: f32, translate_z: f32) -> Vec<f32> {
    vec![
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        translate_x, translate_y, translate_z, 1.0,
    ]
}

#[rustfmt::skip]
pub fn rotate_x(rad_angle: f32) -> Vec<f32> {
    let cosine = rad_angle.cos();
    let sine = rad_angle.sin();
    vec![
        1.0, 0.0, 0.0, 0.0,
        0.0, cosine, sine, 0.0, 
        0.0, -sine, cosine, 0.0,
        0.0, 0.0, 0.0, 1.0, 
    ]
}

#[rustfmt::skip]
pub fn rotate_y(rad_angle: f32) -> Vec<f32> {
    let cosine = rad_angle.cos();
    let sine = rad_angle.sin();
    vec![
        cosine, 0.0, -sine, 0.0,
        0.0, 1.0, 0.0, 0.0, 
        sine, 0.0, cosine, 0.0,
        0.0, 0.0, 0.0, 1.0, 
    ]
}

#[rustfmt::skip]
pub fn rotate_z(rad_angle: f32) -> Vec<f32> {
    let cosine = rad_angle.cos();
    let sine = rad_angle.sin();
    vec![
         cosine, sine, 0.0, 0.0,
         -sine, cosine, 0.0, 0.0,
         0.0, 0.0, 1.0, 0.0, 
         0.0, 0.0, 0.0, 1.0, 
    ]
}

#[rustfmt::skip]
pub fn scale(scale_x: f32, scale_y: f32, scale_z: f32) -> Vec<f32> {
    vec![
        scale_x, 0.0, 0.0, 0.0,
        0.0, scale_y, 0.0, 0.0,
        0.0, 0.0, scale_z, 0.0,
         0.0, 0.0, 0.0, 1.0,
    ]
}

#[rustfmt::skip]
pub fn identity_matrix() -> Vec<f32> {
    vec![
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

#[rustfmt::skip]
pub fn orthographic_projection(left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> Vec<f32> {
    vec![
        2.0/(right - left),               0.0,                            0.0,                       0.0,
        0.0,                              2.0/(top - bottom),             0.0,                       0.0,
        0.0,                              0.0,                            1.0/(z_near - z_far),      0.0,
        (right + left) / (left - right), (top + bottom) / (bottom - top), z_near / (z_near - z_far), 1.0,
    ]
}

#[rustfmt::skip]
pub fn perspective_projection(fov_rad: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Vec<f32> {
    let f = (PI * 0.5 - 0.5 * fov_rad).tan();
    let range_inverse = 1.0 / (z_near - z_far);

    vec![
        f / aspect_ratio,   0.0,    0.0,                                0.0,
        0.0,                f,      0.0,                                0.0,
        0.0,                0.0,    z_far * range_inverse,              -1.0,
        0.0,                0.0,    z_near * z_far * range_inverse,     0.0
    ]
}

#[allow(clippy::all)]
pub fn multiply(a: &[f32], b: &[f32]) -> Vec<f32> {
    debug_assert!(a.len() == b.len());

    let b00 = b[0 * 4 + 0];
    let b01 = b[0 * 4 + 1];
    let b02 = b[0 * 4 + 2];
    let b03 = b[0 * 4 + 3];
    let b10 = b[1 * 4 + 0];
    let b11 = b[1 * 4 + 1];
    let b12 = b[1 * 4 + 2];
    let b13 = b[1 * 4 + 3];
    let b20 = b[2 * 4 + 0];
    let b21 = b[2 * 4 + 1];
    let b22 = b[2 * 4 + 2];
    let b23 = b[2 * 4 + 3];
    let b30 = b[3 * 4 + 0];
    let b31 = b[3 * 4 + 1];
    let b32 = b[3 * 4 + 2];
    let b33 = b[3 * 4 + 3];
    let a00 = a[0 * 4 + 0];
    let a01 = a[0 * 4 + 1];
    let a02 = a[0 * 4 + 2];
    let a03 = a[0 * 4 + 3];
    let a10 = a[1 * 4 + 0];
    let a11 = a[1 * 4 + 1];
    let a12 = a[1 * 4 + 2];
    let a13 = a[1 * 4 + 3];
    let a20 = a[2 * 4 + 0];
    let a21 = a[2 * 4 + 1];
    let a22 = a[2 * 4 + 2];
    let a23 = a[2 * 4 + 3];
    let a30 = a[3 * 4 + 0];
    let a31 = a[3 * 4 + 1];
    let a32 = a[3 * 4 + 2];
    let a33 = a[3 * 4 + 3];

    vec![
        b00 * a00 + b01 * a10 + b02 * a20 + b03 * a30,
        b00 * a01 + b01 * a11 + b02 * a21 + b03 * a31,
        b00 * a02 + b01 * a12 + b02 * a22 + b03 * a32,
        b00 * a03 + b01 * a13 + b02 * a23 + b03 * a33,
        b10 * a00 + b11 * a10 + b12 * a20 + b13 * a30,
        b10 * a01 + b11 * a11 + b12 * a21 + b13 * a31,
        b10 * a02 + b11 * a12 + b12 * a22 + b13 * a32,
        b10 * a03 + b11 * a13 + b12 * a23 + b13 * a33,
        b20 * a00 + b21 * a10 + b22 * a20 + b23 * a30,
        b20 * a01 + b21 * a11 + b22 * a21 + b23 * a31,
        b20 * a02 + b21 * a12 + b22 * a22 + b23 * a32,
        b20 * a03 + b21 * a13 + b22 * a23 + b23 * a33,
        b30 * a00 + b31 * a10 + b32 * a20 + b33 * a30,
        b30 * a01 + b31 * a11 + b32 * a21 + b33 * a31,
        b30 * a02 + b31 * a12 + b32 * a22 + b33 * a32,
        b30 * a03 + b31 * a13 + b32 * a23 + b33 * a33,
    ]
}

pub fn inverse(a: &[f32]) -> Vec<f32> {
    let mut inverse = vec![0.0; a.len()];

    let m00 = a[0 * 4 + 0];
    let m01 = a[0 * 4 + 1];
    let m02 = a[0 * 4 + 2];
    let m03 = a[0 * 4 + 3];
    let m10 = a[1 * 4 + 0];
    let m11 = a[1 * 4 + 1];
    let m12 = a[1 * 4 + 2];
    let m13 = a[1 * 4 + 3];
    let m20 = a[2 * 4 + 0];
    let m21 = a[2 * 4 + 1];
    let m22 = a[2 * 4 + 2];
    let m23 = a[2 * 4 + 3];
    let m30 = a[3 * 4 + 0];
    let m31 = a[3 * 4 + 1];
    let m32 = a[3 * 4 + 2];
    let m33 = a[3 * 4 + 3];

    let tmp0 = m22 * m33;
    let tmp1 = m32 * m23;
    let tmp2 = m12 * m33;
    let tmp3 = m32 * m13;
    let tmp4 = m12 * m23;
    let tmp5 = m22 * m13;
    let tmp6 = m02 * m33;
    let tmp7 = m32 * m03;
    let tmp8 = m02 * m23;
    let tmp9 = m22 * m03;
    let tmp10 = m02 * m13;
    let tmp11 = m12 * m03;
    let tmp12 = m20 * m31;
    let tmp13 = m30 * m21;
    let tmp14 = m10 * m31;
    let tmp15 = m30 * m11;
    let tmp16 = m10 * m21;
    let tmp17 = m20 * m11;
    let tmp18 = m00 * m31;
    let tmp19 = m30 * m01;
    let tmp20 = m00 * m21;
    let tmp21 = m20 * m01;
    let tmp22 = m00 * m11;
    let tmp23 = m10 * m01;

    let t0 = (tmp0 * m11 + tmp3 * m21 + tmp4 * m31) - (tmp1 * m11 + tmp2 * m21 + tmp5 * m31);
    let t1 = (tmp1 * m01 + tmp6 * m21 + tmp9 * m31) - (tmp0 * m01 + tmp7 * m21 + tmp8 * m31);
    let t2 = (tmp2 * m01 + tmp7 * m11 + tmp10 * m31) - (tmp3 * m01 + tmp6 * m11 + tmp11 * m31);
    let t3 = (tmp5 * m01 + tmp8 * m11 + tmp11 * m21) - (tmp4 * m01 + tmp9 * m11 + tmp10 * m21);

    let d = 1.0 / (m00 * t0 + m10 * t1 + m20 * t2 + m30 * t3);

    inverse[0] = d * t0;
    inverse[1] = d * t1;
    inverse[2] = d * t2;
    inverse[3] = d * t3;

    inverse[4] =
        d * ((tmp1 * m10 + tmp2 * m20 + tmp5 * m30) - (tmp0 * m10 + tmp3 * m20 + tmp4 * m30));
    inverse[5] =
        d * ((tmp0 * m00 + tmp7 * m20 + tmp8 * m30) - (tmp1 * m00 + tmp6 * m20 + tmp9 * m30));
    inverse[6] =
        d * ((tmp3 * m00 + tmp6 * m10 + tmp11 * m30) - (tmp2 * m00 + tmp7 * m10 + tmp10 * m30));
    inverse[7] =
        d * ((tmp4 * m00 + tmp9 * m10 + tmp10 * m20) - (tmp5 * m00 + tmp8 * m10 + tmp11 * m20));

    inverse[8] =
        d * ((tmp12 * m13 + tmp15 * m23 + tmp16 * m33) - (tmp13 * m13 + tmp14 * m23 + tmp17 * m33));
    inverse[9] =
        d * ((tmp13 * m03 + tmp18 * m23 + tmp21 * m33) - (tmp12 * m03 + tmp19 * m23 + tmp20 * m33));
    inverse[10] =
        d * ((tmp14 * m03 + tmp19 * m13 + tmp22 * m33) - (tmp15 * m03 + tmp18 * m13 + tmp23 * m33));
    inverse[11] =
        d * ((tmp17 * m03 + tmp20 * m13 + tmp23 * m23) - (tmp16 * m03 + tmp21 * m13 + tmp22 * m23));

    inverse[12] =
        d * ((tmp14 * m22 + tmp17 * m32 + tmp13 * m12) - (tmp16 * m32 + tmp12 * m12 + tmp15 * m22));
    inverse[13] =
        d * ((tmp20 * m32 + tmp12 * m02 + tmp19 * m22) - (tmp18 * m22 + tmp21 * m32 + tmp13 * m02));
    inverse[14] =
        d * ((tmp18 * m12 + tmp23 * m32 + tmp15 * m02) - (tmp22 * m32 + tmp14 * m02 + tmp19 * m12));
    inverse[15] =
        d * ((tmp22 * m22 + tmp16 * m02 + tmp21 * m12) - (tmp20 * m12 + tmp23 * m22 + tmp17 * m02));
    return inverse;
}

pub fn cross(lhs: &[f32], rhs: &[f32]) -> Vec<f32> {
    debug_assert!(lhs.len() == rhs.len());
    let mut result = vec![0.0; lhs.len()];

    let t0 = lhs[1] * rhs[2] - lhs[2] * rhs[1];
    let t1 = lhs[2] * rhs[0] - lhs[0] * rhs[2];
    let t2 = lhs[0] * rhs[1] - lhs[1] * rhs[0];

    result[0] = t0;
    result[1] = t1;
    result[2] = t2;

    return result;
}

pub fn subtract(lhs: &[f32], rhs: &[f32]) -> Vec<f32> {
    debug_assert!(lhs.len() == rhs.len());
    let mut result = vec![0.0; lhs.len()];

    result[0] = lhs[0] - rhs[0];
    result[1] = lhs[1] - rhs[1];
    result[2] = lhs[2] - rhs[2];

    return result;
}

pub fn normalize(vector: &[f32]) -> Vec<f32> {
    debug_assert!(vector.len() == 3);
    let mut result = vec![0.0; vector.len()];

    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    // make sure we don't divide by 0.
    if length > 0.00001 {
        result[0] = vector[0] / length;
        result[1] = vector[1] / length;
        result[2] = vector[2] / length;
    } else {
        result[0] = 0.0;
        result[1] = 0.0;
        result[2] = 0.0;
    }

    return result;
}

pub fn cameraAim(eye: &[f32], target: &[f32], up: &[f32]) -> Vec<f32> {
    debug_assert!(eye.len() == target.len() && target.len() == up.len() && eye.len() == 3);
    let mut result = vec![0.0; 16];

    let z_axis = normalize(&subtract(eye, target));
    let x_axis = normalize(&cross(up, &z_axis));
    let y_axis = normalize(&cross(&z_axis, &x_axis));

    result[0] = x_axis[0];
    result[1] = x_axis[1];
    result[2] = x_axis[2];
    result[3] = 0.0;
    result[4] = y_axis[0];
    result[5] = y_axis[1];
    result[6] = y_axis[2];
    result[7] = 0.0;
    result[8] = z_axis[0];
    result[9] = z_axis[1];
    result[10] = z_axis[2];
    result[11] = 0.0;
    result[12] = eye[0];
    result[13] = eye[1];
    result[14] = eye[2];
    result[15] = 1.0;

    return result;
}

pub struct Wgpu {
    pub inner_size: PhysicalSize<u32>,
    pub adapter: Adapter,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub vertex_count: u32,
    pub object_data: (Buffer, BindGroup),
}

impl Wgpu {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let inner_size = window.inner_size();
        let surface = instance.create_surface(window).unwrap();
        // Request an adapter that can support our surface
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create logical device and command queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("gpu_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device");
        println!("Prepared device: {:?}", device);

        // Configure surface
        let config = surface
            .get_default_config(&adapter, inner_size.width, inner_size.height)
            .unwrap();
        surface.configure(&device, &config);

        // Load the shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        // Vertex buffer
        #[rustfmt::skip]
        let f_char_vertices: Vec<f32> = vec![
            // left column
            -50.0,  75.0,  15.0,
            -20.0,  75.0,  15.0,
            -50.0, -75.0,  15.0,
            -20.0, -75.0,  15.0,
           // top rung
            -20.0,  75.0,  15.0,
             50.0,  75.0,  15.0,
            -20.0,  45.0,  15.0,
             50.0,  45.0,  15.0,
           // middle rung
            -20.0,  15.0,  15.0,
             20.0,  15.0,  15.0,
            -20.0, -15.0,  15.0,
             20.0, -15.0,  15.0,
           // left column back
            -50.0,  75.0, -15.0,
            -20.0,  75.0, -15.0,
            -50.0, -75.0, -15.0,
            -20.0, -75.0, -15.0,
           // top rung back
            -20.0,  75.0, -15.0,
             50.0,  75.0, -15.0,
            -20.0,  45.0, -15.0,
             50.0,  45.0, -15.0,
           // middle rung back
            -20.0,  15.0, -15.0,
             20.0,  15.0, -15.0,
            -20.0, -15.0, -15.0,
             20.0, -15.0, -15.0,
        ];

        // Vertex indices
        let f_char_indices: Vec<u32> = vec![
            0, 2, 1, 2, 3, 1, // left column
            4, 6, 5, 6, 7, 5, // top run
            8, 10, 9, 10, 11, 9, // middle run
            12, 13, 14, 14, 13, 15, // left column back
            16, 17, 18, 18, 17, 19, // top run back
            20, 21, 22, 22, 21, 23, // middle run back
            0, 5, 12, 12, 5, 17, // top
            5, 7, 17, 17, 7, 19, // top rung right
            6, 18, 7, 18, 19, 7, // top rung bottom
            6, 8, 18, 18, 8, 20, // between top and middle rung
            8, 9, 20, 20, 9, 21, // middle rung top
            9, 11, 21, 21, 11, 23, // middle rung right
            10, 22, 11, 22, 23, 11, // middle rung bottom
            10, 3, 22, 22, 3, 15, // stem right
            2, 14, 3, 14, 15, 3, // bottom
            0, 12, 2, 12, 14, 2, // left
        ];
        // Each vertex index corresponds to a vertex to be used which is
        // more than the number of vertices we have.
        let vertex_count = f_char_indices.len() as u32;

        let quad_colors: Vec<u8> = vec![
            200, 70, 120, // left column front
            200, 70, 120, // top rung front
            200, 70, 120, // middle rung front
            80, 70, 200, // left column back
            80, 70, 200, // top rung back
            80, 70, 200, // middle rung back
            70, 200, 210, // top
            160, 160, 220, // top rung right
            90, 130, 110, // top rung bottom
            200, 200, 70, // between top and middle rung
            210, 100, 70, // middle rung top
            210, 160, 70, // middle rung right
            70, 180, 210, // middle rung bottom
            100, 70, 210, // stem right
            76, 210, 100, // bottom
            140, 210, 80, // left
        ];

        let vertex_data = {
            f_char_indices
                .iter()
                .enumerate()
                .flat_map(|(i, index)| {
                    let start_vertex_index = (index * 3) as usize;
                    let vertex_iter = (start_vertex_index..start_vertex_index + 3)
                        .map(|vertex_index| f_char_vertices[vertex_index]);

                    let start_color_index = (i / 6 | 0) as usize * 3;
                    let color = f32::from_le_bytes([
                        quad_colors[start_color_index],
                        quad_colors[start_color_index + 1],
                        quad_colors[start_color_index + 2],
                        255,
                    ]);

                    vertex_iter.chain([color])
                })
                .collect::<Vec<f32>>()
        };

        let vertex_data = vertex_data
            .iter()
            .flat_map(|entry| entry.to_le_bytes())
            .collect::<Vec<u8>>();

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            size: vertex_data.len() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, &vertex_data);

        // Bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline_descriptor"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: 4 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: wgpu::VertexFormat::Unorm8x4,
                            offset: 12,
                            shader_location: 1,
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
            multiview: None,
            cache: None,
        });

        // Preparing for rendering
        let object_data = {
            // Uniform buffer
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("uniforms"),
                // uniforms have to be padded to a multiple of 8
                size: 16 * 4_u64, // matrix * float32 + padding
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Create bind group
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("uniforms"),
                layout: &bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None, // use whole buffer
                    }),
                }],
            });

            (uniform_buffer, bind_group)
        };

        Wgpu {
            inner_size,
            adapter,
            surface,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            vertex_count,
            object_data,
        }
    }

    pub fn render(&mut self, camera_eye: &[f32; 3]) {
        // Create render texture
        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swap-chain texture");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = self.device.create_texture(&TextureDescriptor {
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

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
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
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // the up vector for the camera
            let up = vec![0.0, 1.0, 0.0];
            // the camera matrix
            let camera = cameraAim(camera_eye, &vec![0.0, 0.0, -120.0], &up);
            // view matrix
            let view_matrix = inverse(&camera);

            let projecton_matrix = perspective_projection(
                PI / 2.0, // PI / 2.0 rad => 90 degrees
                self.inner_size.width as f32 / self.inner_size.height as f32,
                1.0,
                2000.0,
            );
            let view_projection_matrix = multiply(&projecton_matrix, &view_matrix);

            let translation = translate(0.0, 0.0, -120.0);
            let rotation_on_y = rotate_y(-PI / 4.0);
            let rotation_on_z = rotate_z(-PI / 4.0);
            let scaling = scale(1.0, 1.0, 1.0);
            // move the origin of the 'F' into the origo
            let translate_origin = translate(-50.0, -75.0, 0.0);
            let matrix = multiply(
                &view_projection_matrix,
                &multiply(
                    &multiply(
                        &multiply(&multiply(&translation, &rotation_on_z), &rotation_on_y),
                        &scaling,
                    ),
                    &translate_origin,
                ),
            );

            let uniforms = matrix
                .iter()
                .flat_map(|entry| entry.to_le_bytes())
                .collect::<Vec<u8>>();

            self.queue.write_buffer(&self.object_data.0, 0, &uniforms);

            render_pass.set_bind_group(0, &self.object_data.1, &[]);
            render_pass.draw(0..self.vertex_count, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
