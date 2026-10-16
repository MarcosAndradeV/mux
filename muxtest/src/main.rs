use muxapp::*;
use muxui::d3::*;
use muxui::*;

struct AppState {
    counter: i32,
    cube_rotation: f32,
    rotation_speed: f32,
    show_wireframe: bool,
    cube_pos: Vector3,
    bg_color: Color,
    alert_pulse: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 0,
            cube_rotation: 0.0,
            rotation_speed: 60.0,
            show_wireframe: true,
            cube_pos: Vector3::zero(),
            bg_color: get_color(0x181818FF),
            alert_pulse: 0.0,
        }
    }
}

struct MainScene {
    orbit_cam: OrbitCamera,
    terrain_mesh: GpuMesh,
}

impl Default for MainScene {
    fn default() -> Self {
        let orbit_cam = OrbitCamera::new(Vector3::zero(), 14.0)
            .with_angles(45.0, 30.0)
            .with_pitch_limits(5.0, 80.0)
            .with_fovy(45.0);

        // Build an indexed GpuMesh (45x45 quads = 2,025 quads = 4,050 triangles)
        let grid_size = 45usize;
        let cell_size = 0.5f32;
        let half = (grid_size as f32 * cell_size) / 2.0;

        let mut vertices = Vec::with_capacity(grid_size * grid_size * 4);
        let mut indices = Vec::with_capacity(grid_size * grid_size * 6);

        for z in 0..grid_size {
            for x in 0..grid_size {
                let x0 = x as f32 * cell_size - half;
                let z0 = z as f32 * cell_size - half;
                let x1 = x0 + cell_size;
                let z1 = z0 + cell_size;

                let c = if (x + z) % 2 == 0 {
                    Color { r: 35, g: 35, b: 40, a: 255 }
                } else {
                    Color { r: 25, g: 25, b: 30, a: 255 }
                };

                let base = vertices.len() as u16;
                vertices.push(Vertex3D::new(Vector3::new(x0, 0.0, z0), c));
                vertices.push(Vertex3D::new(Vector3::new(x1, 0.0, z0), c));
                vertices.push(Vertex3D::new(Vector3::new(x1, 0.0, z1), c));
                vertices.push(Vertex3D::new(Vector3::new(x0, 0.0, z1), c));

                indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            }
        }

        let terrain_mesh = GpuMesh::from_indexed(&vertices, &indices);

        Self {
            orbit_cam,
            terrain_mesh,
        }
    }
}

impl Scene<AppState> for MainScene {
    fn on_enter(&mut self, state: &mut AppState, style: &Style) {
        state.bg_color = style.get_val_or(&["background_color"], get_color(0x181818FF));
        state.rotation_speed = style.get_val_or(&["scene3d", "rotation_speed"], state.rotation_speed);
    }

    fn update(&mut self, state: &mut AppState, input: &Input, _style: &Style, dt: f32) -> SceneAction {
        if input.is_action_pressed("pause") {
            return SceneAction::Push("pause".to_string());
        }

        state.cube_rotation += state.rotation_speed * dt;
        state.alert_pulse = (state.alert_pulse + dt * 4.0) % (std::f32::consts::PI * 2.0);

        // OrbitCamera rotation via continuous mouse axes
        let mouse_yaw = input.get_axis("yaw") * 40.0 * dt;
        let mouse_pitch = input.get_axis("pitch") * 40.0 * dt;
        self.orbit_cam.rotate(mouse_yaw, mouse_pitch);

        // 1D Axis control via Input mapper (WASD)
        let dx = input.get_axis("horizontal");
        let dz = input.get_axis("vertical");
        let move_speed = 6.0;

        let delta = Vector3::new(dx, 0.0, dz) * (move_speed * dt);
        state.cube_pos = state.cube_pos + delta;

        SceneAction::None
    }

    fn draw_3d(&mut self, state: &AppState) {
        let camera = self.orbit_cam.to_camera_3d();
        draw_3d(camera, |d3| {
            d3.draw_grid(20, 1.0);

            // Retained GPU-resident indexed mesh rendered in 1 draw call
            d3.draw_mesh(&self.terrain_mesh);

            // Matrix transformation for animated rotating cube
            d3.push_matrix(|ctx| {
                ctx.translate(state.cube_pos);
                ctx.rotate(state.cube_rotation, Vector3::new(0.0, 1.0, 0.5));
                ctx.draw_cube(Vector3::zero(), 2.0, 2.0, 2.0, GOLD);
                if state.show_wireframe {
                    ctx.draw_cube_wires(Vector3::zero(), 2.05, 2.05, 2.05, LIME);
                }
            });

            // Satellite sphere revolving around the cube
            let angle_rad = state.cube_rotation * 0.03;
            let sat_offset = Vector3::new(4.0 * angle_rad.cos(), 1.0, 4.0 * angle_rad.sin());
            let sat_pos = state.cube_pos + sat_offset;
            d3.draw_sphere(sat_pos, 0.5, SKYBLUE);
            if state.show_wireframe {
                d3.draw_sphere_wires(sat_pos, 0.52, 8, 8, WHITE);
            }

            // Origin coordinate axes
            d3.draw_line_3d(Vector3::zero(), Vector3::new(2.0, 0.0, 0.0), RED);
            d3.draw_line_3d(Vector3::zero(), Vector3::new(0.0, 2.0, 0.0), GREEN);
            d3.draw_line_3d(Vector3::zero(), Vector3::new(0.0, 0.0, 2.0), BLUE);
        });
    }

    fn draw_2d(&mut self, state: &mut AppState, style: &Style) -> SceneAction {
        TextElement::new("MUX 0.4.0 - 3D Engine & Systems").place(style, "title");
        TextElement::new("Retained GpuMesh - Composite Overlay Scenes - OrbitCamera - Sliders & Checkboxes").place(style, "subtitle");

        // Dynamic alert badge with GSS class toggling
        let is_warning = state.cube_pos.length() > 5.0;
        let badge_text = if is_warning {
            format!("ALERT: HIGH OFFSET [{:.1}m]", state.cube_pos.length())
        } else {
            "STATUS: NOMINAL (Within 5m)".to_string()
        };
        let badge = TextElement::new(badge_text)
            .with_class(if is_warning { "warning" } else { "normal" });
        badge.place(style, "alert_badge");

        let display = format!(
            "Count: {} | Cube Pos: ({:.1}, {:.1}) | GpuMesh Tris: {}",
            state.counter,
            state.cube_pos.x,
            state.cube_pos.z,
            self.terrain_mesh.triangle_count()
        );
        TextElement::new(display).place(style, "counter_display");

        // Action Buttons (No Cell<bool> flag dance needed!)
        let pause_btn = Button::new("Pause (P)");
        let inc10 = Button::new("+10");
        let dec5 = Button::new("-5");
        let reset = Button::new("Reset");

        StackLayout::new(&[
            ("btn_pause", &pause_btn),
            ("btn_inc10", &inc10),
            ("btn_dec5", &dec5),
            ("btn_reset", &reset),
        ])
        .place(style, "btn_row");

        if inc10.clicked() {
            state.counter += 10;
        }
        if dec5.clicked() {
            state.counter -= 5;
        }
        if reset.clicked() {
            state.counter = 0;
            state.cube_pos = Vector3::zero();
        }

        // Interactive Slider & Checkbox widgets
        let speed_slider = Slider::new(0.0, 240.0, state.rotation_speed)
            .with_step(5.0)
            .with_label("Rot Speed");
        speed_slider.place(style, "speed_slider");
        speed_slider.apply(&mut state.rotation_speed);

        Checkbox::bind("Show Wireframe Overlay", &mut state.show_wireframe, style, "chk_wireframe");

        // 3D In-World to Screen HUD Projected Waypoint
        let camera = self.orbit_cam.to_camera_3d();
        let angle_rad = state.cube_rotation * 0.03;
        let sat_offset = Vector3::new(4.0 * angle_rad.cos(), 1.0, 4.0 * angle_rad.sin());
        let sat_pos = state.cube_pos + sat_offset;
        if let Some(screen_pos) = get_virtual_world_to_screen(&camera, sat_pos) {
            let pulse_factor = (state.alert_pulse.sin() + 1.0) / 2.0;
            let bracket_color = GOLD.lerp(WHITE, pulse_factor);
            draw_text("[ SAT-1 ]", screen_pos.x as i32 - 28, screen_pos.y as i32 - 20, 14, bracket_color);
            draw_circle(screen_pos.x as i32, screen_pos.y as i32, 3.0, bracket_color);
        }

        // Edge-anchored footer
        RectangleElement.place(style, "footer_bar");
        TextElement::new("WASD: Move Cube | Mouse: Orbit Look | P/Esc: Pause Overlay | F5: Reload GSS").place(style, "footer_text");

        if pause_btn.clicked() {
            return SceneAction::Push("pause".to_string());
        }
        SceneAction::None
    }
}

#[derive(Default)]
struct PauseOverlayScene;

impl Scene<AppState> for PauseOverlayScene {
    fn layer_mode(&self) -> SceneLayerMode {
        // Composite overlay: renders on top of the frozen 3D MainScene!
        SceneLayerMode::Overlay
    }

    fn update(&mut self, _state: &mut AppState, input: &Input, _style: &Style, _dt: f32) -> SceneAction {
        if input.is_action_pressed("pause") {
            return SceneAction::Pop;
        }
        SceneAction::None
    }

    fn draw_2d(&mut self, state: &mut AppState, style: &Style) -> SceneAction {
        // Semi-transparent backdrop over the frozen 3D world
        draw_rectangle_rec(get_virtual_screen_rect(), get_color(0x000000AA));

        TextElement::new("SIMULATION PAUSED").place(style, "title");
        TextElement::new("Composite SceneLayerMode::Overlay rendered over frozen 3D scene").place(style, "subtitle");

        let resume_btn = Button::new("Resume (P)");
        let reset_pos_btn = Button::new("Reset Pos");

        StackLayout::new(&[
            ("btn_pause", &resume_btn),
            ("btn_pause", &reset_pos_btn),
        ])
        .place(style, "btn_row");

        if resume_btn.clicked() {
            return SceneAction::Pop;
        }
        if reset_pos_btn.clicked() {
            state.cube_pos = Vector3::zero();
            return SceneAction::Pop;
        }
        SceneAction::None
    }
}

fn main() {
    App::init(800, 600, "MUX 0.4.0 - Planetary & 3D Simulation Engine", AppState::default)
        .disable_exit_key()
        .set_style_file("./data/test.gss")
        .set_fps(60)
        // Configure input action and axis mapping
        .register_ui_action("pause", [KEY_P, KEY_ESCAPE])
        .register_axis("horizontal", KEY_D, KEY_A)
        .register_axis("vertical", KEY_S, KEY_W)
        // Continuous mouse axis mapping
        .register_mouse_axis("yaw", MouseAxis::X, 0.05)
        .register_mouse_axis("pitch", MouseAxis::Y, 0.05)
        // Register scenes
        .add_scene("main", MainScene::default())
        .add_scene("pause", PauseOverlayScene)
        .set_active_scene("main")
        .run();
}
