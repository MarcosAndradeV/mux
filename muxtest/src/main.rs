use std::cell::Cell;
use muxapp::*;
use muxui::d3::*;
use muxui::*;

#[derive(Default)]
struct AppState {
    counter: i32,
    cube_rotation: f32,
    cube_pos: Vector3,
    cam_yaw: f32,
    cam_pitch: f32,
    bg_color: Color,
    alert_pulse: f32,
}

struct MainScene {
    camera: Camera3D,
    terrain_mesh: Vec<Vertex3D>,
    pause_requested: Cell<bool>,
}

impl Default for MainScene {
    fn default() -> Self {
        let camera = Camera3DBuilder::new()
            .position(Vector3::new(0.0, 8.0, 12.0))
            .target(Vector3::zero())
            .up(Vector3::new(0.0, 1.0, 0.0))
            .fovy(45.0)
            .build();

        // Generate a large procedural mesh with over 10,000 vertices
        // to verify that draw_trimesh_buffered safely chunks and checks batch limits
        let mut terrain_mesh = Vec::new();
        let grid_size = 45; // 45x45 quads = 2025 quads = 4050 triangles = 12,150 vertices (> 8,192 batch limit!)
        let cell_size = 0.5f32;
        let half = (grid_size as f32 * cell_size) / 2.0;

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

                let v00 = Vector3::new(x0, 0.0, z0);
                let v10 = Vector3::new(x1, 0.0, z0);
                let v11 = Vector3::new(x1, 0.0, z1);
                let v01 = Vector3::new(x0, 0.0, z1);

                // Triangle 1
                terrain_mesh.push(Vertex3D::new(v00, c));
                terrain_mesh.push(Vertex3D::new(v10, c));
                terrain_mesh.push(Vertex3D::new(v11, c));

                // Triangle 2
                terrain_mesh.push(Vertex3D::new(v00, c));
                terrain_mesh.push(Vertex3D::new(v11, c));
                terrain_mesh.push(Vertex3D::new(v01, c));
            }
        }

        Self {
            camera,
            terrain_mesh,
            pause_requested: Cell::new(false),
        }
    }
}

impl Scene<AppState> for MainScene {
    fn on_enter(&mut self, state: &mut AppState, style: &Style) {
        state.bg_color = style.get_val_or(&["background_color"], get_color(0x181818FF));
    }

    fn update(&mut self, state: &mut AppState, input: &Input, style: &Style, dt: f32) -> SceneAction {
        if input.is_action_pressed("pause") || self.pause_requested.take() {
            return SceneAction::Push("pause".to_string());
        }

        // Live-tuned GSS parameters via FromGssValue
        let rot_speed = style.get_val_or(&["scene3d", "rotation_speed"], 60.0);
        state.cube_rotation += rot_speed * dt;
        state.alert_pulse = (state.alert_pulse + dt * 4.0) % (std::f32::consts::PI * 2.0);

        // Continuous mouse input combined with keyboard axes
        let mouse_yaw = input.get_axis("yaw");
        let mouse_pitch = input.get_axis("pitch");
        state.cam_yaw += mouse_yaw * 40.0 * dt;
        state.cam_pitch = (state.cam_pitch + mouse_pitch * 40.0 * dt).clamp(-30.0, 60.0);

        // 1D Axis control via Input mapper (WASD / Arrow Keys)
        let dx = input.get_axis("horizontal");
        let dz = input.get_axis("vertical");
        let move_speed = 6.0;

        // Native Vector3 arithmetic: Vector3 + Vector3 * scalar
        let delta = Vector3::new(dx, 0.0, dz) * (move_speed * dt);
        state.cube_pos = state.cube_pos + delta;

        // Dynamic camera orbital position
        let rad_yaw = (state.cam_yaw + 45.0).to_radians();
        let rad_pitch = (state.cam_pitch + 30.0).to_radians();
        let dist = 14.0;
        self.camera.position = Vector3::new(
            dist * rad_pitch.cos() * rad_yaw.sin(),
            dist * rad_pitch.sin().max(0.1),
            dist * rad_pitch.cos() * rad_yaw.cos(),
        );

        SceneAction::None
    }

    fn draw_3d(&mut self, state: &AppState) {
        draw_3d(self.camera, |d3| {
            d3.draw_grid(20, 1.0);

            // Large procedural terrain mesh (>10,000 vertices) chunked safely without buffer overflow
            d3.draw_trimesh_buffered(&self.terrain_mesh);

            // Matrix transformation for animated rotating cube
            d3.push_matrix(|ctx| {
                ctx.translate(state.cube_pos);
                ctx.rotate(state.cube_rotation, Vector3::new(0.0, 1.0, 0.5));
                ctx.draw_cube(Vector3::zero(), 2.0, 2.0, 2.0, GOLD);
                ctx.draw_cube_wires(Vector3::zero(), 2.05, 2.05, 2.05, LIME);
            });

            // Satellite sphere revolving around the cube
            let angle_rad = state.cube_rotation * 0.03;
            let sat_offset = Vector3::new(4.0 * angle_rad.cos(), 1.0, 4.0 * angle_rad.sin());
            let sat_pos = state.cube_pos + sat_offset;
            d3.draw_sphere(sat_pos, 0.5, SKYBLUE);
            d3.draw_sphere_wires(sat_pos, 0.52, 8, 8, WHITE);

            // Origin coordinate axes
            d3.draw_line_3d(Vector3::zero(), Vector3::new(2.0, 0.0, 0.0), RED);
            d3.draw_line_3d(Vector3::zero(), Vector3::new(0.0, 2.0, 0.0), GREEN);
            d3.draw_line_3d(Vector3::zero(), Vector3::new(0.0, 0.0, 2.0), BLUE);
        });
    }

    fn draw_2d(&mut self, state: &mut AppState, style: &Style) {
        TextElement::new("MUX 0.3.0 - 3D Engine & Systems").place(style, "title");
        TextElement::new("Batch-Safe Mesh (12k Vertices) - Mouse Delta Axes - Dynamic GSS Classes - 3D HUD Projection").place(style, "subtitle");

        // Dynamic alert badge with GSS class toggling and Color::lerp
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
            "Count: {} | Cube Pos: ({:.1}, {:.1}) | Vertices: {}",
            state.counter, state.cube_pos.x, state.cube_pos.z, self.terrain_mesh.len()
        );
        TextElement::new(display).place(style, "counter_display");

        // Action Buttons
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

        if pause_btn.clicked() {
            self.pause_requested.set(true);
        }
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

        // 3D In-World to Screen HUD Projected Waypoint
        let angle_rad = state.cube_rotation * 0.03;
        let sat_offset = Vector3::new(4.0 * angle_rad.cos(), 1.0, 4.0 * angle_rad.sin());
        let sat_pos = state.cube_pos + sat_offset;
        if let Some(screen_pos) = get_virtual_world_to_screen(&self.camera, sat_pos) {
            let pulse_factor = (state.alert_pulse.sin() + 1.0) / 2.0;
            let bracket_color = GOLD.lerp(WHITE, pulse_factor);
            draw_text("[ SAT-1 ]", screen_pos.x as i32 - 28, screen_pos.y as i32 - 20, 14, bracket_color);
            draw_circle(screen_pos.x as i32, screen_pos.y as i32, 3.0, bracket_color);
        }

        // Edge-anchored footer
        RectangleElement.place(style, "footer_bar");
        TextElement::new("WASD: Move Cube | Right-Drag/Mouse: Look | P: Pause | F5: Reload GSS").place(style, "footer_text");
    }
}

#[derive(Default)]
struct PauseOverlayScene {
    resume_requested: Cell<bool>,
}

impl Scene<AppState> for PauseOverlayScene {
    fn update(&mut self, _state: &mut AppState, input: &Input, _style: &Style, _dt: f32) -> SceneAction {
        if input.is_action_pressed("pause") || self.resume_requested.take() {
            return SceneAction::Pop;
        }
        SceneAction::None
    }

    fn draw_2d(&mut self, state: &mut AppState, style: &Style) {
        // Semi-transparent backdrop overlay
        draw_rectangle_rec(get_virtual_screen_rect(), get_color(0x000000AA));

        TextElement::new("SIMULATION PAUSED").place(style, "title");
        TextElement::new("Modal scene overlay stacked on top of 3D scene").place(style, "subtitle");

        let resume_btn = Button::new("Resume (P)");
        let reset_pos_btn = Button::new("Reset Pos");

        StackLayout::new(&[
            ("btn_pause", &resume_btn),
            ("btn_pause", &reset_pos_btn),
        ])
        .place(style, "btn_row");

        if resume_btn.clicked() {
            self.resume_requested.set(true);
        }
        if reset_pos_btn.clicked() {
            state.cube_pos = Vector3::zero();
            self.resume_requested.set(true);
        }
    }
}

fn main() {
    App::init(800, 600, "MUX 0.3.0 - Planetary & 3D Simulation Engine", AppState::default)
        .set_style_file("./data/test.gss")
        .set_fps(60)
        // Configure input action and axis mapping
        .register_action("pause", [KEY_P, KEY_ESCAPE])
        .register_axis("horizontal", KEY_D, KEY_A)
        .register_axis("vertical", KEY_S, KEY_W)
        // Continuous mouse axis mapping
        .register_mouse_axis("yaw", MouseAxis::X, 0.05)
        .register_mouse_axis("pitch", MouseAxis::Y, 0.05)
        // Register scenes
        .add_scene("main", MainScene::default())
        .add_scene("pause", PauseOverlayScene::default())
        .set_active_scene("main")
        .run();
}
