use std::sync::Arc;

use crate::{normal_debug, normal_debug_wireframe, skybox, textured_draw};
use graphic::camera::Camera;
use wgpu::{Device, ExperimentalFeatures, Queue, Surface};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct RenderServer {
    // TODO remove this as well,
    // only relevant for the camera which
    // should become a node later anyways
    inner_size: PhysicalSize<u32>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    // Rendering
    mesh_id: u32,
    mesh_cache: std::collections::HashMap<u32, MeshBuffer>,
    global_uniform_buffer: wgpu::Buffer,
    // Rendering pipelines
    textured: textured_draw::Textured,
    normal_debug: normal_debug::NormalDebug,
    normal_debug_wireframe: normal_debug_wireframe::NormalDebugWireframe,
    skybox: skybox::Skybox,
}

impl RenderServer {
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
                experimental_features: ExperimentalFeatures::disabled(),
            })
            .await
            .expect("Failed to create device");
        println!("Prepared device: {device:?}",);

        // Configure surface
        let config = surface
            .get_default_config(&adapter, inner_size.width, inner_size.height)
            .unwrap();
        surface.configure(&device, &config);

        let swapchain_capabilities = surface.get_capabilities(&adapter);
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

        let mesh_cache = std::collections::HashMap::<u32, MeshBuffer>::new();

        let textured = textured_draw::Textured::new(&device, swapchain_format.into());
        let normal_debug = normal_debug::NormalDebug::new(&device, swapchain_format.into());
        let normal_debug_wireframe =
            normal_debug_wireframe::NormalDebugWireframe::new(&device, swapchain_format.into());
        let skybox = skybox::Skybox::new(&device, &queue, swapchain_format.into());

        RenderServer {
            inner_size,
            surface,
            device,
            queue,
            global_uniform_buffer,
            mesh_id: 0,
            mesh_cache,
            textured,
            normal_debug,
            normal_debug_wireframe,
            skybox,
        }
    }

    pub fn load_mesh(&mut self, mesh: &mesh::Mesh) {
        let vertex_data = mesh
            .indices()
            .iter()
            .flat_map(|index| {
                let vertex = &mesh.vertices()[*index as usize];

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

        let vertex_buffer = upload_vertex_buffer(&self.device, &self.queue, &vertex_data);
        let vertex_count = mesh.indices().len() as u32;

        self.mesh_cache.insert(
            self.mesh_id,
            MeshBuffer {
                vertex_buffer,
                vertex_count,
            },
        );
        self.mesh_id += 1;
    }

    pub fn load_scene(&mut self) {
        // Notify "textured" pipeline about the instances it needs to draw
        // PLANE
        let mut mesh_buffer = self.mesh_cache.get(&10u32).unwrap();
        let plane_texture = crate::texture::load_texture_plane(&self.device, &self.queue);
        self.textured.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![textured_draw::Instance {
                model_matrix: graphic::transform::translate(0.0, -1.0, 0.0)
                    * graphic::transform::scale(50.0, 1.0, 50.0),
            }],
            vec![textured_draw::TextureInstance {
                texture_scale: 50.0,
            }],
            plane_texture,
        );

        mesh_buffer = self.mesh_cache.get(&11u32).unwrap();
        let cube_texture = crate::texture::load_texture_cube(&self.device, &self.queue);
        self.textured.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![textured_draw::Instance {
                model_matrix: graphic::identity_matrix(),
            }],
            vec![textured_draw::TextureInstance { texture_scale: 1.0 }],
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

        mesh_buffer = self.mesh_cache.get(&0u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            suzanne_flat_967_instances,
        );

        // SUZANNE flat 967 messed up normals
        mesh_buffer = self.mesh_cache.get(&1u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(-10.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        );

        // SUZANNE smooth 967
        mesh_buffer = self.mesh_cache.get(&2u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }],
        );

        // SUZANNE smooth 967 messed up normals
        mesh_buffer = self.mesh_cache.get(&3u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(-5.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        );

        // Utah teapot flat 7k
        mesh_buffer = self.mesh_cache.get(&4u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(0.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Utah teapot smooth 7k
        mesh_buffer = self.mesh_cache.get(&5u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(5.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Utah teapot smooth 116k
        mesh_buffer = self.mesh_cache.get(&6u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(10.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Stanford dragon flat 17k
        mesh_buffer = self.mesh_cache.get(&7u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Stanford dragon smooth 17k
        mesh_buffer = self.mesh_cache.get(&8u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Stanford dragon smooth 700k
        mesh_buffer = self.mesh_cache.get(&9u32).unwrap();
        self.normal_debug.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug::Instance {
                model_matrix: graphic::transform::translate(10.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Notify "normal_debug_wireframe" about the instances it needs to draw
        mesh_buffer = self.mesh_cache.get(&0u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }],
        );

        // SUZANNE flat 967 messed up normals
        mesh_buffer = self.mesh_cache.get(&1u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(-10.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        );

        // SUZANNE smooth 967
        mesh_buffer = self.mesh_cache.get(&2u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -5.0)
                    * graphic::transform::scale(2.0, 2.0, 2.0),
            }],
        );

        // SUZANNE smooth 967 messed up normals
        mesh_buffer = self.mesh_cache.get(&3u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(-5.0, 0.0, -5.0)
                    * graphic::transform::scale(1.0, 1.0, 1.0),
            }],
        );

        // Utah teapot flat 7k
        mesh_buffer = self.mesh_cache.get(&4u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(0.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Utah teapot smooth 7k
        mesh_buffer = self.mesh_cache.get(&5u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(5.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Utah teapot smooth 116k
        mesh_buffer = self.mesh_cache.get(&6u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(10.0, -0.2, -10.0)
                    * graphic::transform::scale(0.5, 0.5, 0.5),
            }],
        );

        // Stanford dragon flat 17k
        mesh_buffer = self.mesh_cache.get(&7u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(0.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Stanford dragon smooth 17k
        mesh_buffer = self.mesh_cache.get(&8u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(5.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        // Stanford dragon smooth 700k
        mesh_buffer = self.mesh_cache.get(&9u32).unwrap();
        self.normal_debug_wireframe.add_entity_instances(
            &self.device,
            &self.global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(10.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );
    }

    pub fn render(&mut self, camera: &Camera, wireframe: bool) {
        // the camera matrix
        let look_at = camera.as_transform_matrix();
        // view matrix
        let view_matrix = look_at;

        let aspect_ratio = self.inner_size.width as f32 / self.inner_size.height as f32;
        let projection_matrix = graphic::transform::perspective_proj_sym_h_fov(
            // PI / 2.0, 90 deg FOV
            (std::f32::consts::PI / 180.0) * 75.0, // 75 degree FOV
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

        self.queue
            .write_buffer(&self.global_uniform_buffer, 0, &global_uniforms);

        // Create render texture
        let frame = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swap-chain texture");
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: frame.texture.size(),
            mip_level_count: 1, // no extra mips, has to be 1
            sample_count: 1,    // no multisampling, so 1
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
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
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
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
                &self.queue,
                &translation_free_view_projection_matrix,
            );

            // Render textured objects
            self.textured.render(
                &mut render_pass,
                &self.queue,
                camera,
                &view_projection_matrix,
            );

            if !wireframe {
                // Render normal debug shaded objects
                self.normal_debug.render(
                    &mut render_pass,
                    &self.queue,
                    camera,
                    &view_matrix,
                    &view_projection_matrix,
                );
            } else {
                // Render normal debug shaded objects in wireframe mode
                self.normal_debug_wireframe.render(
                    &mut render_pass,
                    &self.queue,
                    camera,
                    &view_matrix,
                    &view_projection_matrix,
                );
            }
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}

#[derive(Debug)]
pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
}

fn upload_vertex_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    vertex_data: &[u8],
) -> wgpu::Buffer {
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("vertex_buffer"),
        size: vertex_data.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, vertex_data);

    vertex_buffer
}
