use graphic::camera::Camera;
use std::{collections, f32::consts::PI};

use crate::{
    normal_debug::{self, NormalDebug},
    normal_debug_wireframe::{self, NormalDebugWireframe},
    skybox::Skybox,
    textured_draw::{self, TextureInstance},
};
use wgpu::{
    Adapter, Device, Operations, Queue, RenderPassDepthStencilAttachment, Surface,
    TextureDescriptor, TextureUsages,
};
use winit::dpi::PhysicalSize;

#[derive(Debug)]
struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
}

// Just load all the meshes that we use in the screen and upload
// them to the GPU, in their own buffers.
fn populate_mesh_cache(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> collections::HashMap<u32, MeshBuffer> {
    let mut mesh_cache = collections::HashMap::<u32, MeshBuffer>::new();

    // SUZANNE flat 967
    let suzanne_flat_967_data = include_str!("../resources/meshes/suzanne_flat_967.obj");
    let suzanne_flat_967 = format::wavefront::Obj::parse(
        suzanne_flat_967_data.lines().map(String::from),
        "Suzanne_flat_967",
    );
    mesh_cache.insert(0, upload_vertex_buffer(device, queue, &suzanne_flat_967));

    // SUZANNE flat 967 messed up normals
    let suzanne_flat_967_messed_up_normals_data =
        include_str!("../resources/meshes/suzanne_flat_967_messed_up_normals.obj");
    let suzanne_flat_967_messed_up_normals = format::wavefront::Obj::parse(
        suzanne_flat_967_messed_up_normals_data
            .lines()
            .map(String::from),
        "Suzanne_flat_967_messed_up_normals",
    );
    mesh_cache.insert(
        1,
        upload_vertex_buffer(device, queue, &suzanne_flat_967_messed_up_normals),
    );

    // SUZANNE smooth 967
    let suzanne_smooth_967_data = include_str!("../resources/meshes/suzanne_smooth_967.obj");
    let suzanne_smooth_967 = format::wavefront::Obj::parse(
        suzanne_smooth_967_data.lines().map(String::from),
        "Suzanne_smooth_967",
    );
    mesh_cache.insert(2, upload_vertex_buffer(device, queue, &suzanne_smooth_967));

    // SUZANNE smooth 967 messed up normals
    let suzanne_smooth_967_messed_up_normals_data =
        include_str!("../resources/meshes/suzanne_smooth_967_messed_up_normals.obj");
    let suzanne_smooth_967_messed_up_normals = format::wavefront::Obj::parse(
        suzanne_smooth_967_messed_up_normals_data
            .lines()
            .map(String::from),
        "Suzanne_smooth_967_messed_up_normals",
    );
    mesh_cache.insert(
        3,
        upload_vertex_buffer(device, queue, &suzanne_smooth_967_messed_up_normals),
    );

    // Utah teapot flat 7k
    let utah_flat_7k_data = include_str!("../resources/meshes/utah_teapot_flat_7k.obj");
    let utah_flat_7k =
        format::wavefront::Obj::parse(utah_flat_7k_data.lines().map(String::from), "Utah_flat_7k");
    mesh_cache.insert(4, upload_vertex_buffer(device, queue, &utah_flat_7k));

    // Utah teapot smooth 7k
    let utah_smooth_7k_data = include_str!("../resources/meshes/utah_teapot_smooth_7k.obj");
    let utah_smooth_7k = format::wavefront::Obj::parse(
        utah_smooth_7k_data.lines().map(String::from),
        "Utah_smooth_7k",
    );
    mesh_cache.insert(5, upload_vertex_buffer(device, queue, &utah_smooth_7k));

    // Utah teapot smooth 116k
    let utah_smooth_116k_data = include_str!("../resources/meshes/utah_teapot_smooth_116k.obj");
    let utah_smooth_116k = format::wavefront::Obj::parse(
        utah_smooth_116k_data.lines().map(String::from),
        "Utah_smooth_116k",
    );
    mesh_cache.insert(6, upload_vertex_buffer(device, queue, &utah_smooth_116k));

    // Stanford dragon flat 17k
    let stanford_dragon_flat_17k_data =
        include_str!("../resources/meshes/stanford_dragon_flat_17k.obj");
    let stanford_dragon_flat_17k = format::wavefront::Obj::parse(
        stanford_dragon_flat_17k_data.lines().map(String::from),
        "Stanford_dragon_flat_17k",
    );
    mesh_cache.insert(
        7,
        upload_vertex_buffer(device, queue, &stanford_dragon_flat_17k),
    );

    // Stanford dragon smooth 17k
    let stanford_dragon_smooth_17k_data =
        include_str!("../resources/meshes/stanford_dragon_smooth_17k.obj");
    let stanford_dragon_smooth_17k = format::wavefront::Obj::parse(
        stanford_dragon_smooth_17k_data.lines().map(String::from),
        "Stanford_dragon_smooth_17k",
    );
    mesh_cache.insert(
        8,
        upload_vertex_buffer(device, queue, &stanford_dragon_smooth_17k),
    );

    // Stanford dragon smooth 700k
    let stanford_dragon_smooth_700k_data =
        include_str!("../resources/meshes/stanford_dragon_smooth_700k.obj");
    let stanford_dragon_smooth_700k = format::wavefront::Obj::parse(
        stanford_dragon_smooth_700k_data.lines().map(String::from),
        "Stanford_dragon_smooth_700k",
    );
    mesh_cache.insert(
        9,
        upload_vertex_buffer(device, queue, &stanford_dragon_smooth_700k),
    );

    // Plane entry
    let plane_mesh = crate::mesh::generate_plane();
    let plane_vertex_data = plane_mesh
        .indices()
        .iter()
        .flat_map(|index| {
            let vertex = &plane_mesh.vertices()[*index as usize];

            vertex
                .position()
                .as_slice()
                .iter()
                .chain(vertex.normal().as_slice().iter().chain([&0.0]))
                .chain(vertex.uv().as_slice().iter())
                .chain([&0.0, &0.0])
                .flat_map(|value| value.to_le_bytes())
        })
        .collect::<Vec<u8>>();
    let vertex_count = plane_mesh.indices().len() as u32;

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vertex_buffer"),
        size: plane_vertex_data.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, &plane_vertex_data);
    mesh_cache.insert(
        10,
        MeshBuffer {
            vertex_buffer,
            vertex_count,
        },
    );

    // Cube entry
    let cube_mesh = crate::mesh::generate_cube();
    let cube_vertex_data = cube_mesh
        .indices()
        .iter()
        .flat_map(|index| {
            let vertex = &cube_mesh.vertices()[*index as usize];

            vertex
                .position()
                .as_slice()
                .iter()
                .chain(vertex.normal().as_slice().iter().chain([&0.0]))
                .chain(vertex.uv().as_slice().iter())
                .chain([&0.0, &0.0])
                .flat_map(|value| value.to_le_bytes())
        })
        .collect::<Vec<u8>>();
    let vertex_count = cube_mesh.indices().len() as u32;

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vertex_buffer"),
        size: cube_vertex_data.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, &cube_vertex_data);
    mesh_cache.insert(
        11,
        MeshBuffer {
            vertex_buffer,
            vertex_count,
        },
    );

    mesh_cache
}

fn upload_vertex_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    object: &format::wavefront::Obj,
) -> MeshBuffer {
    let vertex_data = object
        .faces()
        .iter()
        .flat_map(|face| {
            let vertices = object.vertices();
            let normals = object.normals();

            face.iter().flat_map(|vertex| {
                let face_vertex = &vertices[vertex.vertex_index() - 1];
                // it is possible that a mesh doesn't contain normals either
                // may have to handle it
                let face_normal = &normals[vertex.normal_index().unwrap() - 1];

                face_vertex
                    .as_slice()
                    .iter()
                    .chain(face_normal.as_slice().iter().chain([&0.0]))
                    .flat_map(|value| value.to_le_bytes())
            })
        })
        .collect::<Vec<u8>>();
    let vertex_count = (object.faces().len() * 3) as u32;

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vertex_buffer"),
        size: vertex_data.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, &vertex_data);

    MeshBuffer {
        vertex_buffer,
        vertex_count,
    }
}

// A Scene should be a structure which manages the lifetimes
// of any mesh, texture, sound, shader that is used in the scene.
// It can handle the hierarchical scene elements, their transformations etc.
//
// This is not the desired Scene as it has no such structures.
// It also contains logic specifying how it should be transformed into
// commands for the GPU. That should be the domain of a completely different class,
// but for the time being it has been moved here as well.
// Mostly to keep things simple.
#[derive(Debug)]
pub struct Scene {
    global_uniform_buffer: wgpu::Buffer,
    textured: textured_draw::Textured,
    normal_debug: NormalDebug,
    normal_debug_wireframe: NormalDebugWireframe,
    skybox: Skybox,
    _mesh_cache: collections::HashMap<u32, MeshBuffer>,
}

impl Scene {
    pub fn new(adapter: &Adapter, surface: &Surface, device: &Device, queue: &Queue) -> Self {
        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        // Global Uniform buffer
        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            // uniforms have to be padded to a multiple of 8
            #[allow(clippy::identity_op)] // for clearer explanation
            size: (16 + 16 + 3 + 1) * 4, // (view matrix, view projection matrix, view_world_position, pad) * float size
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut textured = textured_draw::Textured::new(device, swapchain_format.into());
        let mut normal_debug = NormalDebug::new(device, swapchain_format.into());
        let mut normal_debug_wireframe = NormalDebugWireframe::new(device, swapchain_format.into());
        let skybox = Skybox::new(device, queue, swapchain_format.into());

        let mesh_cache = populate_mesh_cache(device, queue);

        // Notify "textured" pipeline about the instances it needs to draw
        // PLANE
        let mut mesh_buffer = mesh_cache.get(&10u32).unwrap();
        let plane_texture = crate::texture::load_texture_plane(device, queue);
        textured.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![textured_draw::Instance {
                model_matrix: graphic::transform::translate(0.0, -1.0, 0.0)
                    * graphic::transform::scale(50.0, 1.0, 50.0),
            }],
            vec![TextureInstance {
                texture_scale: 50.0,
            }],
            plane_texture,
        );

        mesh_buffer = mesh_cache.get(&11u32).unwrap();
        let cube_texture = crate::texture::load_texture_cube(device, queue);
        textured.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![textured_draw::Instance {
                model_matrix: graphic::identity_matrix(),
            }],
            vec![TextureInstance { texture_scale: 1.0 }],
            cube_texture,
        );

        // Notify "normal_debug" about the instances it needs to draw
        // TODO little dirty with the hand managed IDs, but that is okay for feeling out
        // the pattern.

        // SUZANNE flat 967
        let suzanne_flat_967_instances = {
            let suzanne_flat_967_instances = vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }];
            // let mut x = -10.0;
            // while x <= 10.0 {
            //     let mut z = -5.0;
            //     while z >= -25.0 {
            //         let mut y = 5.0;
            //         while y <= 15.0 {
            //             suzanne_flat_967_instances.push(normal_debug::Instance {
            //                 model_matrix: graphic::transform::translate(x, y, z)
            //                     * graphic::transform::scale(1.0, 1.0, 1.0),
            //             });
            //             y += 5.0;
            //         }
            //         z -= 5.0;
            //     }
            //     x += 5.0;
            // }
            suzanne_flat_967_instances
        };

        mesh_buffer = mesh_cache.get(&0u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            suzanne_flat_967_instances,
        );

        // SUZANNE flat 967 messed up normals
        mesh_buffer = mesh_cache.get(&1u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(-10.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        );

        // SUZANNE smooth 967
        mesh_buffer = mesh_cache.get(&2u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }],
        );

        // SUZANNE smooth 967 messed up normals
        mesh_buffer = mesh_cache.get(&3u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(-5.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        );

        // Utah teapot flat 7k
        mesh_buffer = mesh_cache.get(&4u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(0.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Utah teapot smooth 7k
        mesh_buffer = mesh_cache.get(&5u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(5.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Utah teapot smooth 116k
        mesh_buffer = mesh_cache.get(&6u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(10.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Stanford dragon flat 17k
        mesh_buffer = mesh_cache.get(&7u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Stanford dragon smooth 17k
        mesh_buffer = mesh_cache.get(&8u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Stanford dragon smooth 700k
        mesh_buffer = mesh_cache.get(&9u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(10.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Notify "normal_debug_wireframe" about the instances it needs to draw
        mesh_buffer = mesh_cache.get(&0u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }],
        );

        // SUZANNE flat 967 messed up normals
        mesh_buffer = mesh_cache.get(&1u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(-10.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        );

        // SUZANNE smooth 967
        mesh_buffer = mesh_cache.get(&2u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }],
        );

        // SUZANNE smooth 967 messed up normals
        mesh_buffer = mesh_cache.get(&3u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(-5.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        );

        // Utah teapot flat 7k
        mesh_buffer = mesh_cache.get(&4u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(0.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Utah teapot smooth 7k
        mesh_buffer = mesh_cache.get(&5u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(5.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Utah teapot smooth 116k
        mesh_buffer = mesh_cache.get(&6u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(10.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Stanford dragon flat 17k
        mesh_buffer = mesh_cache.get(&7u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Stanford dragon smooth 17k
        mesh_buffer = mesh_cache.get(&8u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Stanford dragon smooth 700k
        mesh_buffer = mesh_cache.get(&9u32).unwrap();
        normal_debug_wireframe.add_entity_instances(
            device,
            &global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(10.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        Self {
            global_uniform_buffer,
            textured,
            normal_debug,
            normal_debug_wireframe,
            skybox,
            _mesh_cache: mesh_cache,
        }
    }

    pub fn render(
        &mut self,
        inner_size: &PhysicalSize<u32>,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
        camera: &Camera,
        wireframe: bool,
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

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });

        // the camera matrix
        let look_at = camera.as_transform_matrix();
        // view matrix
        let view_matrix = look_at;

        let aspect_ratio = inner_size.width as f32 / inner_size.height as f32;
        let projection_matrix = graphic::transform::perspective_proj_sym_h_fov(
            // PI / 2.0, 90 deg FOV
            (PI / 180.0) * 75.0, // 75 degree FOV
            aspect_ratio,
            -0.05,
            -4000.0,
        );

        // Render the rest
        let view_projection_matrix = projection_matrix * view_matrix;

        // It does not matter if it is rendered first or last, because
        // the skybox is at Z value 1.0 in NDC.
        // No other draw call, should write if the depth value equals 1.0.
        let translation_free_view_matrix = {
            let mut tmp = view_matrix;
            tmp[(0, 3)] = 0.0;
            tmp[(1, 3)] = 0.0;
            tmp[(2, 3)] = 0.0;
            tmp
        };
        let translation_free_view_projection_matrix =
            projection_matrix * translation_free_view_matrix;

        let transposed_view_matrix = view_matrix.transpose();
        let transposed_view_projection_matrix = view_projection_matrix.transpose();

        // Update Uniforms
        let global_uniforms = transposed_view_matrix
            .as_slices()
            .iter()
            .flatten()
            .flat_map(|entry| entry.to_le_bytes())
            .chain(
                transposed_view_projection_matrix
                    .as_slices()
                    .iter()
                    .flatten()
                    .flat_map(|entry| entry.to_le_bytes()),
            )
            .chain(
                [camera.eye()[0], camera.eye()[1], camera.eye()[2], 0.0]
                    .iter()
                    .flat_map(|entry| entry.to_le_bytes()),
            )
            .collect::<Vec<u8>>();
        queue.write_buffer(&self.global_uniform_buffer, 0, &global_uniforms);

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

            // Render skybox
            self.skybox.render(
                &mut render_pass,
                queue,
                &translation_free_view_projection_matrix,
            );

            // Render textured objects
            self.textured
                .render(&mut render_pass, queue, camera, &view_projection_matrix);

            if !wireframe {
                // Render normal debug shaded objects
                self.normal_debug.render(
                    &mut render_pass,
                    queue,
                    camera,
                    &view_matrix,
                    &view_projection_matrix,
                );
            } else {
                // Render normal debug shaded objects in wireframe mode
                self.normal_debug_wireframe.render(
                    &mut render_pass,
                    queue,
                    camera,
                    &view_matrix,
                    &view_projection_matrix,
                );
            }
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
