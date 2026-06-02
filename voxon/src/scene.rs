use graphic::camera::Camera;
use std::{f32::consts::PI, time::Duration};

use crate::{
    normal_debug::NormalDebug, normal_debug_instanced::InstancedNormalDebug,
    normal_debug_wireframe::NormalDebugWireframe, skybox::Skybox, textured_draw::TexturedEntities,
};
use wgpu::{
    Adapter, Device, Operations, Queue, RenderPassDepthStencilAttachment, Surface,
    TextureDescriptor, TextureUsages,
};
use winit::dpi::PhysicalSize;

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
    textured_entities: TexturedEntities,
    normal_debug: NormalDebug,
    normal_debug_wireframe: NormalDebugWireframe,
    normal_debug_instanced: InstancedNormalDebug,
    skybox: Skybox,
}

impl Scene {
    pub fn new(adapter: &Adapter, surface: &Surface, device: &Device, queue: &Queue) -> Self {
        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let textured_entities = TexturedEntities::new(device, queue, swapchain_format.into());
        let normal_debug = NormalDebug::new(device, queue, swapchain_format.into());
        let normal_debug_wireframe =
            NormalDebugWireframe::new(device, queue, swapchain_format.into());
        let normal_debug_instanced =
            InstancedNormalDebug::new(device, queue, swapchain_format.into());
        let skybox = Skybox::new(device, queue, swapchain_format.into());

        Self {
            textured_entities,
            normal_debug,
            normal_debug_wireframe,
            normal_debug_instanced,
            skybox,
        }
    }

    pub fn simulate(&mut self, delta_t: Duration) {
        // World simulation.
        self.textured_entities.simulate(delta_t);
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

            // Render skybox
            // It does not matter if it is rendered first or last, beacause
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
            self.skybox.render(
                &mut render_pass,
                queue,
                &translation_free_view_projection_matrix,
            );

            // Render the rest
            let view_projection_matrix = projection_matrix * view_matrix;

            // Render textured objects
            self.textured_entities
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

            // Render instanced normal debug shaded dragons
            // self.normal_debug_instanced.render(
            //     &mut render_pass,
            //     queue,
            //     camera,
            //     &view_matrix,
            //     &view_projection_matrix,
            // );
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
