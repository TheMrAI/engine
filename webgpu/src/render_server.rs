use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::{cube_map, normal_debug, normal_debug_wireframe, skybox, textured_draw};
use graphic::camera::Camera;
use scene::MeshNode;
use wgpu::{Device, ExperimentalFeatures, Queue, Surface, Texture};
use winit::{dpi::PhysicalSize, window::Window};

// A bit redundant at the moment but we need
// to hide webgpu API related types from the interface.
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
}

type MeshInstanceCache = std::collections::HashMap<u32, Vec<Rc<RefCell<MeshNode>>>>;

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
    next_mesh_id: u32,
    mesh_cache: std::collections::HashMap<u32, MeshBuffer>,
    next_shader_id: u32,
    shader_cache: std::collections::HashMap<u32, wgpu::ShaderModule>,
    global_uniform_buffer: wgpu::Buffer,
    next_texture_id: u32,
    texture_cache: std::collections::HashMap<u32, Texture>,
    // Rendering pipelines
    render_entries: std::collections::HashMap<u32, MeshInstanceCache>,
    textured: textured_draw::Textured,
    normal_debug: normal_debug::NormalDebug,
    normal_debug_wireframe: normal_debug_wireframe::NormalDebugWireframe,
    cube_map: cube_map::CubeMap,
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
        let shader_cache = std::collections::HashMap::<u32, wgpu::ShaderModule>::new();
        let texture_cache = std::collections::HashMap::<u32, Texture>::new();
        let render_entries = std::collections::HashMap::<
            u32,
            std::collections::HashMap<u32, Vec<Rc<RefCell<MeshNode>>>>,
        >::new();

        let textured = textured_draw::Textured::new(&device, swapchain_format.into());
        let normal_debug = normal_debug::NormalDebug::new(&device, swapchain_format.into());
        let normal_debug_wireframe =
            normal_debug_wireframe::NormalDebugWireframe::new(&device, swapchain_format.into());
        let cube_map = cube_map::CubeMap::new(&device, swapchain_format.into());
        let skybox = skybox::Skybox::new(&device, &queue, swapchain_format.into());

        RenderServer {
            inner_size,
            surface,
            device,
            queue,
            global_uniform_buffer,
            next_mesh_id: 0,
            mesh_cache,
            next_shader_id: 0,
            shader_cache,
            next_texture_id: 0,
            texture_cache,
            render_entries,
            textured,
            normal_debug,
            normal_debug_wireframe,
            cube_map,
            skybox,
        }
    }

    // TODO Gotta figure out how the server may identify which mesh it already knows about,
    // so caching can actually cache.
    // Have a sneaking suspicion that the Scene/Nodes will provide with some intel.
    pub fn load_mesh(&mut self, mesh: &mesh::Mesh) -> u32 {
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

        let mesh_id = self.next_mesh_id;
        self.mesh_cache.insert(
            mesh_id,
            MeshBuffer {
                vertex_buffer,
                vertex_count,
            },
        );

        self.next_mesh_id += 1;
        mesh_id
    }

    pub fn load_shader(&mut self, shader_code: std::borrow::Cow<str>) -> u32 {
        let shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader_code"),
                source: wgpu::ShaderSource::Wgsl(shader_code),
            });

        let shader_id = self.next_shader_id;
        self.shader_cache.insert(shader_id, shader_module);

        self.next_shader_id += 1;
        shader_id
    }

    pub fn load_texture(&mut self, texture_buffer: &[&[u8]], dimensions: Dimensions) -> u32 {
        let dimensions = wgpu::Extent3d {
            width: dimensions.width,
            height: dimensions.height,
            depth_or_array_layers: dimensions.depth_or_array_layers,
        };

        let texture = crate::texture::upload_texture(
            &self.device,
            &self.queue,
            dimensions,
            texture_buffer,
            true,
        );

        let texture_id = self.next_texture_id;
        self.texture_cache.insert(texture_id, texture);

        self.next_texture_id += 1;
        texture_id
    }

    // TODO Eventually this should not take a node at all, but a reference or a pointer to the Node.
    // The Node is maintained by the Engine in the Scene graph.
    // The RenderServer merely has to access the nodes when appropriate to read the necessary values.
    pub fn schedule_render(&mut self, mesh_node: Rc<RefCell<MeshNode>>) {
        if mesh_node.borrow().shader_id == 0 {
            self.normal_debug.schedule(mesh_node);
        } else {
            let pipeline = self
                .render_entries
                .entry(mesh_node.borrow().shader_id)
                .or_default();
            let instances = pipeline.entry(mesh_node.borrow().mesh_id).or_default();
            instances.push(mesh_node);
        }
    }

    pub fn render(&mut self, camera: &Camera) {
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

            let translation_free_view_matrix = {
                let mut tmp = view_matrix;
                tmp[(0, 3)] = 0.0;
                tmp[(1, 3)] = 0.0;
                tmp[(2, 3)] = 0.0;
                tmp
            };
            let translation_free_view_projection_matrix =
                projection_matrix * translation_free_view_matrix;

            // Render skybox
            self.skybox.render(
                &mut render_pass,
                &self.queue,
                &translation_free_view_projection_matrix,
            );

            self.normal_debug.render(
                &mut render_pass,
                &self.mesh_cache,
                &self.device,
                &self.queue,
                &view_matrix,
                &view_projection_matrix,
                &self.global_uniform_buffer,
            );

            for (shader_id, mesh_group) in &self.render_entries {
                for (mesh_id, mesh_instances) in mesh_group {
                    match *shader_id {
                        1 => {
                            let mesh_buffer = self.mesh_cache.get(mesh_id).unwrap();
                            let instances = mesh_instances
                                .iter()
                                .map(|instance| instance.borrow().model_matrix)
                                .collect::<Vec<_>>();

                            self.normal_debug_wireframe.render(
                                &mut render_pass,
                                &self.device,
                                &self.queue,
                                &view_matrix,
                                &view_projection_matrix,
                                &self.global_uniform_buffer,
                                &instances,
                                &mesh_buffer.vertex_buffer,
                                mesh_buffer.vertex_count,
                            );
                        }
                        2 => {
                            let mesh_buffer = self.mesh_cache.get(mesh_id).unwrap();
                            // TODO very dirty hack!!!
                            // It turns out that for different pipelines you have to instance differently.
                            // For the debug pipelines the mesh_id is enough, but for the textured pipeline
                            // both the mesh_id and the texture itself has to take part in the instancing.
                            // Which cannot be supported by the rudimentary organizer mechanism.
                            // It seems we really need to have the concept of a node, and based on that,
                            // each pipeline may extract the identifiers it needs to properly group them.
                            // We aren't just there yet, so care is required.
                            let texture = self
                                .texture_cache
                                .get(&mesh_instances.first().unwrap().borrow().texture_id.unwrap())
                                .unwrap();
                            let instances = mesh_instances
                                .iter()
                                .map(|instance| {
                                    (
                                        instance.borrow().model_matrix,
                                        instance.borrow().texture_scale.unwrap(),
                                    )
                                })
                                .collect::<Vec<_>>();

                            self.textured.render(
                                &mut render_pass,
                                &self.device,
                                &self.queue,
                                &view_matrix,
                                &view_projection_matrix,
                                &self.global_uniform_buffer,
                                &instances,
                                &mesh_buffer.vertex_buffer,
                                mesh_buffer.vertex_count,
                                texture,
                            );
                        }
                        3 => {
                            let mesh_buffer = self.mesh_cache.get(mesh_id).unwrap();
                            // TODO very dirty hack as above!!
                            let texture = self
                                .texture_cache
                                .get(&mesh_instances.first().unwrap().borrow().texture_id.unwrap())
                                .unwrap();
                            let instances = mesh_instances
                                .iter()
                                .map(|instance| instance.borrow().model_matrix)
                                .collect::<Vec<_>>();

                            self.cube_map.render(
                                &mut render_pass,
                                &self.device,
                                &self.queue,
                                &view_matrix,
                                &view_projection_matrix,
                                &self.global_uniform_buffer,
                                &instances,
                                &mesh_buffer.vertex_buffer,
                                mesh_buffer.vertex_count,
                                texture,
                            );
                        }
                        _ => unimplemented!("Bra no such shader!"),
                    }
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();

        // Purge all scheduled entries for this rendering cycle
        // TODO should not be necessary after the pipelines are able to cache the
        // rendering requests
        self.render_entries.clear();
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
