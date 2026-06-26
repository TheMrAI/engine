use graphic::camera::Camera;

use crate::{
    normal_debug::{self, NormalDebug},
    normal_debug_wireframe::{self, NormalDebugWireframe},
    skybox::Skybox,
    textured_draw::{self, TextureInstance},
};
use wgpu::{Device, Queue};

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
    textured: textured_draw::Textured,
    normal_debug: NormalDebug,
    normal_debug_wireframe: NormalDebugWireframe,
    skybox: Skybox,
}

impl Scene {
    pub fn new(
        swapchain_format: &wgpu::TextureFormat,
        device: &Device,
        queue: &Queue,
        global_uniform_buffer: &wgpu::Buffer,
        mesh_cache: &std::collections::HashMap<u32, crate::web_gpu_render_servers::MeshBuffer>,
    ) -> Self {
        let mut textured = textured_draw::Textured::new(device, (*swapchain_format).into());
        let mut normal_debug = NormalDebug::new(device, (*swapchain_format).into());
        let mut normal_debug_wireframe =
            NormalDebugWireframe::new(device, (*swapchain_format).into());
        let skybox = Skybox::new(device, queue, (*swapchain_format).into());

        // Notify "textured" pipeline about the instances it needs to draw
        // PLANE
        let mut mesh_buffer = mesh_cache.get(&10u32).unwrap();
        let plane_texture = crate::texture::load_texture_plane(device, queue);
        textured.add_entity_instances(
            device,
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            suzanne_flat_967_instances,
        );

        // SUZANNE flat 967 messed up normals
        mesh_buffer = mesh_cache.get(&1u32).unwrap();
        normal_debug.add_entity_instances(
            device,
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
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
            global_uniform_buffer,
            &mesh_buffer.vertex_buffer,
            mesh_buffer.vertex_count,
            vec![normal_debug_wireframe::Instance {
                model_matrix: graphic::transform::translate(10.0, 0.0, -15.0)
                    * graphic::transform::scale(18.0, 18.0, 18.0),
            }],
        );

        Self {
            textured,
            normal_debug,
            normal_debug_wireframe,
            skybox,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        view_matrix: &lina::matrix::Matrix<f32, 4, 4>,
        view_projection_matrix: &lina::matrix::Matrix<f32, 4, 4>,
        translation_free_view_projection_matrix: &lina::matrix::Matrix<f32, 4, 4>,
        queue: &Queue,
        camera: &Camera,
        wireframe: bool,
    ) {
        // Render skybox
        self.skybox
            .render(render_pass, queue, translation_free_view_projection_matrix);

        // Render textured objects
        self.textured
            .render(render_pass, queue, camera, view_projection_matrix);

        if !wireframe {
            // Render normal debug shaded objects
            self.normal_debug.render(
                render_pass,
                queue,
                camera,
                view_matrix,
                view_projection_matrix,
            );
        } else {
            // Render normal debug shaded objects in wireframe mode
            self.normal_debug_wireframe.render(
                render_pass,
                queue,
                camera,
                view_matrix,
                view_projection_matrix,
            );
        }
    }
}
