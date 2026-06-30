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
        // SIMULATING SCENE BY CONSTRUCTING IT BY HAND
        // SUZANNE flat 967
        let suzanne_flat_967_data =
            include_str!("../../voxon/resources/meshes/suzanne_flat_967.obj");
        let suzanne_flat_967 = format::wavefront::Obj::parse(
            suzanne_flat_967_data.lines().map(String::from),
            "Suzanne_flat_967",
        );
        rendering_api.load_mesh(&suzanne_flat_967.try_into().unwrap());

        // SUZANNE flat 967 messed up normals
        let suzanne_flat_967_messed_up_normals_data =
            include_str!("../../voxon/resources/meshes/suzanne_flat_967_messed_up_normals.obj");
        let suzanne_flat_967_messed_up_normals = format::wavefront::Obj::parse(
            suzanne_flat_967_messed_up_normals_data
                .lines()
                .map(String::from),
            "Suzanne_flat_967_messed_up_normals",
        );
        rendering_api.load_mesh(&suzanne_flat_967_messed_up_normals.try_into().unwrap());

        // SUZANNE smooth 967
        let suzanne_smooth_967_data =
            include_str!("../../voxon/resources/meshes/suzanne_smooth_967.obj");
        let suzanne_smooth_967 = format::wavefront::Obj::parse(
            suzanne_smooth_967_data.lines().map(String::from),
            "Suzanne_smooth_967",
        );
        rendering_api.load_mesh(&suzanne_smooth_967.try_into().unwrap());

        // SUZANNE smooth 967 messed up normals
        let suzanne_smooth_967_messed_up_normals_data =
            include_str!("../../voxon/resources/meshes/suzanne_smooth_967_messed_up_normals.obj");
        let suzanne_smooth_967_messed_up_normals = format::wavefront::Obj::parse(
            suzanne_smooth_967_messed_up_normals_data
                .lines()
                .map(String::from),
            "Suzanne_smooth_967_messed_up_normals",
        );
        rendering_api.load_mesh(&suzanne_smooth_967_messed_up_normals.try_into().unwrap());

        // Utah teapot flat 7k
        let utah_flat_7k_data =
            include_str!("../../voxon/resources/meshes/utah_teapot_flat_7k.obj");
        let utah_flat_7k = format::wavefront::Obj::parse(
            utah_flat_7k_data.lines().map(String::from),
            "Utah_flat_7k",
        );
        rendering_api.load_mesh(&utah_flat_7k.try_into().unwrap());

        // Utah teapot smooth 7k
        let utah_smooth_7k_data =
            include_str!("../../voxon/resources/meshes/utah_teapot_smooth_7k.obj");
        let utah_smooth_7k = format::wavefront::Obj::parse(
            utah_smooth_7k_data.lines().map(String::from),
            "Utah_smooth_7k",
        );
        rendering_api.load_mesh(&utah_smooth_7k.try_into().unwrap());

        // Utah teapot smooth 116k
        let utah_smooth_116k_data =
            include_str!("../../voxon/resources/meshes/utah_teapot_smooth_116k.obj");
        let utah_smooth_116k = format::wavefront::Obj::parse(
            utah_smooth_116k_data.lines().map(String::from),
            "Utah_smooth_116k",
        );
        rendering_api.load_mesh(&utah_smooth_116k.try_into().unwrap());

        // Stanford dragon flat 17k
        let stanford_dragon_flat_17k_data =
            include_str!("../../voxon/resources/meshes/stanford_dragon_flat_17k.obj");
        let stanford_dragon_flat_17k = format::wavefront::Obj::parse(
            stanford_dragon_flat_17k_data.lines().map(String::from),
            "Stanford_dragon_flat_17k",
        );
        rendering_api.load_mesh(&stanford_dragon_flat_17k.try_into().unwrap());

        // Stanford dragon smooth 17k
        let stanford_dragon_smooth_17k_data =
            include_str!("../../voxon/resources/meshes/stanford_dragon_smooth_17k.obj");
        let stanford_dragon_smooth_17k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_17k_data.lines().map(String::from),
            "Stanford_dragon_smooth_17k",
        );
        rendering_api.load_mesh(&stanford_dragon_smooth_17k.try_into().unwrap());

        // Stanford dragon smooth 700k
        let stanford_dragon_smooth_700k_data =
            include_str!("../../voxon/resources/meshes/stanford_dragon_smooth_700k.obj");
        let stanford_dragon_smooth_700k = format::wavefront::Obj::parse(
            stanford_dragon_smooth_700k_data.lines().map(String::from),
            "Stanford_dragon_smooth_700k",
        );
        rendering_api.load_mesh(&stanford_dragon_smooth_700k.try_into().unwrap());

        // Plane entry
        let plane_mesh = mesh::generate_plane();
        rendering_api.load_mesh(&plane_mesh);

        // Cube entry
        let cube_mesh = mesh::generate_cube();
        rendering_api.load_mesh(&cube_mesh);

        // Should not happen here
        rendering_api.load_scene();

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

        self.rendering_api.render(&self.camera, self.wireframe);
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
