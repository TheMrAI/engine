use graphic::camera::Camera;
use webgpu::RenderServer;

#[derive(Debug)]
pub struct Game {
    rendering_api: RenderServer,
    wireframe: bool,
    camera: Camera,
    prev_render_time: std::time::Instant,
    prev_mouse_motion_time: std::time::Instant,
    navigation_speed: f32, // speed in m/s
    frametimes: frametime::Sampler<1024>,
    elapsed_time: std::time::Duration,
}

impl Game {
    pub fn new(mut rendering_api: RenderServer) -> Self {
        // SIMULATING SCENE ELEMENTS BY CONSTRUCTING IT BY HAND
        let debug_shader_id = rendering_api.load_shader(std::borrow::Cow::Borrowed(include_str!(
            "../../webgpu/src/normal_debug.wgsl"
        )));
        let debug_shader_wireframe_id = rendering_api.load_shader(std::borrow::Cow::Borrowed(
            include_str!("../../webgpu/src/normal_debug_wireframe.wgsl"),
        ));

        // SUZANNE flat 967
        let suzanne_flat_967_data =
            include_str!("../../voxon/resources/meshes/suzanne_flat_967.obj");
        let suzanne_flat_967 = format::wavefront::Obj::parse(
            suzanne_flat_967_data.lines().map(String::from),
            "Suzanne_flat_967",
        );
        let mut mesh_id = rendering_api.load_mesh(&suzanne_flat_967.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(0.0, 0.0, -5.0)
                * graphic::transform::scale(2.0, 2.0, 2.0),
            shader_id: debug_shader_id,
            texture_id: None,
            texture_scale: None,
        });
        // Instancing load
        // let mut x = -10.0;
        // while x <= 10.0 {
        //     let mut z = -5.0;
        //     while z >= -25.0 {
        //         let mut y = 5.0;
        //         while y <= 15.0 {
        //             rendering_api.schedule_render(scene::MeshNode {
        //                 mesh_id,
        //                 model_matrix: graphic::transform::translate(x, y, z)
        //                     * graphic::transform::scale(1.0, 1.0, 1.0),
        //                 shader_id: debug_shader_id,
        //                 texture_id: None,
        //                 texture_scale: None,
        //             });
        //             y += 5.0;
        //         }
        //         z -= 5.0;
        //     }
        //     x += 5.0;
        // }

        // SUZANNE flat 967 messed up normals
        let suzanne_flat_967_messed_up_normals_data =
            include_str!("../../voxon/resources/meshes/suzanne_flat_967_messed_up_normals.obj");
        let suzanne_flat_967_messed_up_normals = format::wavefront::Obj::parse(
            suzanne_flat_967_messed_up_normals_data
                .lines()
                .map(String::from),
            "Suzanne_flat_967_messed_up_normals",
        );
        mesh_id = rendering_api.load_mesh(&suzanne_flat_967_messed_up_normals.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(-10.0, 0.0, -5.0)
                * graphic::transform::scale(1.0, 1.0, 1.0),
            shader_id: debug_shader_id,
            texture_id: None,
            texture_scale: None,
        });

        // SUZANNE smooth 967
        let suzanne_smooth_967_data =
            include_str!("../../voxon/resources/meshes/suzanne_smooth_967.obj");
        let suzanne_smooth_967 = format::wavefront::Obj::parse(
            suzanne_smooth_967_data.lines().map(String::from),
            "Suzanne_smooth_967",
        );
        mesh_id = rendering_api.load_mesh(&suzanne_smooth_967.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(5.0, 0.0, -5.0)
                * graphic::transform::scale(2.0, 2.0, 2.0),
            shader_id: debug_shader_id,
            texture_id: None,
            texture_scale: None,
        });

        // SUZANNE smooth 967 messed up normals
        let suzanne_smooth_967_messed_up_normals_data =
            include_str!("../../voxon/resources/meshes/suzanne_smooth_967_messed_up_normals.obj");
        let suzanne_smooth_967_messed_up_normals = format::wavefront::Obj::parse(
            suzanne_smooth_967_messed_up_normals_data
                .lines()
                .map(String::from),
            "Suzanne_smooth_967_messed_up_normals",
        );
        mesh_id =
            rendering_api.load_mesh(&suzanne_smooth_967_messed_up_normals.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(-5.0, 0.0, -5.0)
                * graphic::transform::scale(1.0, 1.0, 1.0),
            shader_id: debug_shader_id,
            texture_id: None,
            texture_scale: None,
        });

        // Utah teapot flat 7k
        let utah_flat_7k_data =
            include_str!("../../voxon/resources/meshes/utah_teapot_flat_7k.obj");
        let utah_flat_7k = format::wavefront::Obj::parse(
            utah_flat_7k_data.lines().map(String::from),
            "Utah_flat_7k",
        );
        mesh_id = rendering_api.load_mesh(&utah_flat_7k.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(0.0, -0.2, -10.0)
                * graphic::transform::scale(0.5, 0.5, 0.5),
            shader_id: debug_shader_id,
            texture_id: None,
            texture_scale: None,
        });

        // Utah teapot smooth 7k
        let utah_smooth_7k_data =
            include_str!("../../voxon/resources/meshes/utah_teapot_smooth_7k.obj");
        let utah_smooth_7k = format::wavefront::Obj::parse(
            utah_smooth_7k_data.lines().map(String::from),
            "Utah_smooth_7k",
        );
        mesh_id = rendering_api.load_mesh(&utah_smooth_7k.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(5.0, -0.2, -10.0)
                * graphic::transform::scale(0.5, 0.5, 0.5),
            shader_id: debug_shader_id,
            texture_id: None,
            texture_scale: None,
        });

        // Utah teapot smooth 116k
        let utah_smooth_116k_data =
            include_str!("../../voxon/resources/meshes/utah_teapot_smooth_116k.obj");
        let utah_smooth_116k = format::wavefront::Obj::parse(
            utah_smooth_116k_data.lines().map(String::from),
            "Utah_smooth_116k",
        );
        mesh_id = rendering_api.load_mesh(&utah_smooth_116k.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(10.0, -0.2, -10.0)
                * graphic::transform::scale(0.5, 0.5, 0.5),
            shader_id: debug_shader_id,
            texture_id: None,
            texture_scale: None,
        });

        // Stanford dragon flat 17k
        let stanford_dragon_flat_17k_data =
            include_str!("../../voxon/resources/meshes/stanford_dragon_flat_17k.obj");
        let stanford_dragon_flat_17k = format::wavefront::Obj::parse(
            stanford_dragon_flat_17k_data.lines().map(String::from),
            "Stanford_dragon_flat_17k",
        );
        mesh_id = rendering_api.load_mesh(&stanford_dragon_flat_17k.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(0.0, 0.0, -15.0)
                * graphic::transform::scale(18.0, 18.0, 18.0),
            shader_id: debug_shader_id,
            texture_id: None,
            texture_scale: None,
        });

        // Stanford dragon smooth 17k
        let stanford_dragon_smooth_17k_data =
            include_str!("../../voxon/resources/meshes/stanford_dragon_smooth_17k.obj");
        let stanford_dragon_smooth_17k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_17k_data.lines().map(String::from),
            "Stanford_dragon_smooth_17k",
        );
        mesh_id = rendering_api.load_mesh(&stanford_dragon_smooth_17k.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(5.0, 0.0, -15.0)
                * graphic::transform::scale(18.0, 18.0, 18.0),
            shader_id: debug_shader_wireframe_id,
            texture_id: None,
            texture_scale: None,
        });

        // Stanford dragon smooth 700k
        let stanford_dragon_smooth_700k_data =
            include_str!("../../voxon/resources/meshes/stanford_dragon_smooth_700k.obj");
        let stanford_dragon_smooth_700k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_700k_data.lines().map(String::from),
            "Stanford_dragon_smooth_700k",
        );
        mesh_id = rendering_api.load_mesh(&stanford_dragon_smooth_700k.try_into().unwrap());
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(10.0, 0.0, -15.0)
                * graphic::transform::scale(18.0, 18.0, 18.0),
            shader_id: debug_shader_wireframe_id,
            texture_id: None,
            texture_scale: None,
        });

        // Textured shader
        let textured_shader_id = rendering_api.load_shader(std::borrow::Cow::Borrowed(
            include_str!("../../webgpu/src/textured_draw.wgsl"),
        ));

        // Plane entry
        let image_data = include_bytes!("../../voxon/resources/textures/texture_01.png");
        let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
        let mut reader = png_decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let frame_info = reader.next_frame(&mut buf).unwrap();
        let bytes = &buf[..frame_info.buffer_size()];

        let dimensions = webgpu::Dimensions {
            width: frame_info.width,
            height: frame_info.height,
            depth_or_array_layers: 1,
        };

        let texture_id = rendering_api.load_texture(&[bytes], dimensions);
        let plane_mesh = mesh::generate_plane();
        mesh_id = rendering_api.load_mesh(&plane_mesh);
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::transform::translate(0.0, -1.0, 0.0)
                * graphic::transform::scale(50.0, 1.0, 50.0),
            shader_id: textured_shader_id,
            texture_id: Some(texture_id),
            texture_scale: Some(50.0),
        });

        // Cube entry
        let image_data = include_bytes!("../../voxon/resources/textures/cube_atlas.png");
        let png_decoder = png::Decoder::new(std::io::Cursor::new(image_data));
        let mut reader = png_decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let frame_info = reader.next_frame(&mut buf).unwrap();
        let bytes = &buf[..frame_info.buffer_size()];

        let dimensions = webgpu::Dimensions {
            width: frame_info.width,
            height: frame_info.height,
            depth_or_array_layers: 1,
        };

        let texture_id = rendering_api.load_texture(&[bytes], dimensions);
        let cube_mesh = mesh::generate_cube();
        mesh_id = rendering_api.load_mesh(&cube_mesh);
        rendering_api.schedule_render(scene::MeshNode {
            mesh_id,
            model_matrix: graphic::identity_matrix(),
            shader_id: textured_shader_id,
            texture_id: Some(texture_id),
            texture_scale: Some(1.0),
        });

        Self {
            rendering_api,
            wireframe: false,
            camera: Camera::default(),
            prev_render_time: std::time::Instant::now(),
            prev_mouse_motion_time: std::time::Instant::now(),
            navigation_speed: 1.0,
            frametimes: frametime::Sampler::new(),
            elapsed_time: std::time::Duration::default(),
        }
    }

    pub fn run_loop(
        &mut self,
        key_state: &std::collections::BTreeMap<winit::keyboard::KeyCode, bool>,
    ) {
        let current_time = std::time::Instant::now();
        let delta_t = current_time.duration_since(self.prev_render_time);

        self.process_input(key_state, delta_t);

        // TODO: This not the proper game loop.
        // That would require looping on simulate, with given time steps until
        // the next frame should be rendered.
        self.simulate();

        self.render(delta_t);

        self.prev_render_time = current_time;
        self.prev_mouse_motion_time = current_time;
    }

    // TODO delta_t or none of the key_state belongs here.
    // Processing input should be the part where raw key presses/mouse events
    // are collected and transformed into internal events, that other parts
    // of the engine can react to.
    // Not this hodgepodge of variables and direct transformations, but for
    // the time being such input events are not in our control.
    fn process_input(
        &mut self,
        key_state: &std::collections::BTreeMap<winit::keyboard::KeyCode, bool>,
        delta_t: std::time::Duration,
    ) {
        let key_left_shift = key_state
            .get(&winit::keyboard::KeyCode::ShiftLeft)
            .cloned()
            .unwrap_or(false);
        let key_right_shift = key_state
            .get(&winit::keyboard::KeyCode::ShiftRight)
            .cloned()
            .unwrap_or(false);
        let key_w = key_state
            .get(&winit::keyboard::KeyCode::KeyW)
            .cloned()
            .unwrap_or(false);
        let key_s = key_state
            .get(&winit::keyboard::KeyCode::KeyS)
            .cloned()
            .unwrap_or(false);
        let key_d = key_state
            .get(&winit::keyboard::KeyCode::KeyD)
            .cloned()
            .unwrap_or(false);
        let key_a = key_state
            .get(&winit::keyboard::KeyCode::KeyA)
            .cloned()
            .unwrap_or(false);
        let key_e = key_state
            .get(&winit::keyboard::KeyCode::KeyE)
            .cloned()
            .unwrap_or(false);
        let key_q = key_state
            .get(&winit::keyboard::KeyCode::KeyQ)
            .cloned()
            .unwrap_or(false);
        let key_v = key_state
            .get(&winit::keyboard::KeyCode::KeyV)
            .cloned()
            .unwrap_or(false);
        let key_c = key_state
            .get(&winit::keyboard::KeyCode::KeyC)
            .cloned()
            .unwrap_or(false);

        let elapsed_s = delta_t.as_secs_f32();
        let speed = if key_left_shift || key_right_shift {
            3.0 * self.navigation_speed * elapsed_s
        } else {
            1.0 * self.navigation_speed * elapsed_s
        };
        // TODO this section doesn't belong here, but eh
        if key_w {
            self.camera.move_on_look_at_vector(speed);
        };
        if key_s {
            self.camera.move_on_look_at_vector(-speed);
        };
        if key_d {
            self.camera.move_on_right_vector(speed);
        };
        if key_a {
            self.camera.move_on_right_vector(-speed);
        };
        if key_e {
            self.camera.move_on_up_vector(speed);
        };
        if key_q {
            self.camera.move_on_up_vector(-speed);
        }

        if key_c {
            self.wireframe = false;
        }
        if key_v {
            self.wireframe = true;
        }
    }

    // We have no physics or AI to simulate just yet, but this is where that would be put.
    // According to more experienced devs, the AI/physics has to run with a stable time
    // delta otherwise many approximations could become unstable.
    fn simulate(&mut self) {}

    fn render(&mut self, delta_t: std::time::Duration) {
        self.frametimes.add_frametime(delta_t.as_nanos());
        self.elapsed_time += delta_t;

        if self.elapsed_time > std::time::Duration::from_secs(1) {
            self.elapsed_time -= std::time::Duration::from_secs(1);
            let stats = self.frametimes.stats();
            println!("{}", stats);
        }

        self.rendering_api.render(&self.camera);
    }

    // TODO: This should belong in the process_input functions.
    // At the moment, showing it there would be too costly though.
    pub fn mouse_wheel(&mut self, dy: f32) {
        // To change the speed we use a logarithm function as
        // those types of inputs fell much more natural.
        // Shift it by 1 to the left so it reaches zero at zero,
        // then flatten the result by half.
        // This way within the range os 0.1 - 30 the user
        // gets finer control on the lower ends and coarser on the
        // higher ends.
        self.navigation_speed += dy * ((self.navigation_speed + 1.0).log2() / 2.0);
        self.navigation_speed = self.navigation_speed.clamp(0.1, 30.0);
    }

    // This is hot garbage, but for now it solves the stuttering rotation issue.
    // A few big problems. The camera state is directly modified on the function call.
    // Speed/motion delta is calculated regarding the previous frame render time but that is
    // incorrect.
    // Let us assume that between two frames the mouse registers 3 movement events.
    // The first will correctly calculate the delta_t from the previous frame, but
    // the rest should calculated only from the previous registered mouse motion event.
    // This is why another prev_mouse_motion_time is necessary.
    pub fn mouse_motion(&mut self, delta: (f64, f64)) {
        const ANGULAR_SPEED: f32 = (std::f32::consts::PI / 180.0) * 600.0;
        let current_time = std::time::Instant::now();
        let delta_t = current_time.duration_since(self.prev_mouse_motion_time);

        let elapsed_s = delta_t.as_secs_f32();
        // Negate all inputs, inverting the movements
        self.camera
            .pitch(-delta.1 as f32 * ANGULAR_SPEED * elapsed_s);
        self.camera.yaw(-delta.0 as f32 * ANGULAR_SPEED * elapsed_s);

        self.prev_mouse_motion_time = current_time;
    }
}
