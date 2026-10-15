use std::cell::Cell;
use muxapp::*;
use muxui::*;

#[derive(Default)]
struct AppState {
    counter: i32,
    cube_rotation: f32,
    cube_pos: Vector3,
    bg_color: Color,
}

struct MainScene {
    camera: Camera3D,
    pause_requested: Cell<bool>,
    counter_delta: Cell<i32>,
    reset_counter: Cell<bool>,
}

impl Default for MainScene {
    fn default() -> Self {
        let camera = Camera3DBuilder::new()
            .position(Vector3::new(0.0, 8.0, 12.0))
            .target(Vector3::zero())
            .up(Vector3::new(0.0, 1.0, 0.0))
            .fovy(45.0)
            .build();
        Self {
            camera,
            pause_requested: Cell::new(false),
            counter_delta: Cell::new(0),
            reset_counter: Cell::new(false),
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

        // Apply any button interactions queued from previous frame
        state.counter += self.counter_delta.take();
        if self.reset_counter.take() {
            state.counter = 0;
        }

        // Live-tuned GSS parameters via FromGssValue
        let rot_speed = style.get_val_or(&["scene3d", "rotation_speed"], 60.0);
        state.cube_rotation += rot_speed * dt;

        // Dynamic camera position override if specified in GSS
        if let Some(cam_pos) = style.get_val::<Vector3>(&["scene3d", "camera_pos"]) {
            self.camera.position = cam_pos;
        }

        // 1D Axis control via Input mapper (WASD / Arrow Keys)
        let dx = input.get_axis("horizontal");
        let dz = input.get_axis("vertical");
        let move_speed = 6.0;

        // Native Vector3 arithmetic: Vector3 + Vector3 * scalar
        let delta = Vector3::new(dx, 0.0, dz) * (move_speed * dt);
        state.cube_pos = state.cube_pos + delta;

        SceneAction::None
    }

    fn draw_3d(&mut self, state: &AppState) {
        draw_3d(self.camera, |d3| {
            d3.draw_grid(16, 1.0);

            // Ground plane
            d3.draw_plane(Vector3::zero(), Vector2::new(16.0, 16.0), get_color(0x202020FF));

            // Matrix transformation for animated rotating cube
            d3.push_matrix(|ctx| {
                ctx.translate(state.cube_pos);
                ctx.rotate(state.cube_rotation, Vector3::new(0.0, 1.0, 0.5));
                ctx.draw_cube(Vector3::zero(), 2.0, 2.0, 2.0, GOLD);
                ctx.draw_cube_wires(Vector3::zero(), 2.05, 2.05, 2.05, LIME);
            });

            // Satellite spheres revolving around the cube
            let angle_rad = state.cube_rotation * 0.03;
            let sat_offset = Vector3::new(3.5 * angle_rad.cos(), 1.0, 3.5 * angle_rad.sin());
            let sat_pos = state.cube_pos + sat_offset;
            d3.draw_sphere(sat_pos, 0.5, SKYBLUE);
            d3.draw_sphere_wires(sat_pos, 0.52, 8, 8, WHITE);

            // Origin coordinate axes
            d3.draw_line_3d(Vector3::zero(), Vector3::new(2.0, 0.0, 0.0), RED);
            d3.draw_line_3d(Vector3::zero(), Vector3::new(0.0, 2.0, 0.0), GREEN);
            d3.draw_line_3d(Vector3::zero(), Vector3::new(0.0, 0.0, 2.0), BLUE);
        });
    }

    fn draw_2d(&mut self, state: &AppState, style: &Style) {
        TextElement::new("MUX 0.2.0 - 3D Engine & Scene Architecture").place(style, "title");
        TextElement::new("Safe RAII 3D - Native Vector3 Math - GSS Typed Binding - Input Mapper").place(style, "subtitle");

        let display = format!(
            "Count: {} | Cube Pos: ({:.1}, {:.1})",
            state.counter, state.cube_pos.x, state.cube_pos.z
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
            self.counter_delta.set(self.counter_delta.get() + 10);
        }
        if dec5.clicked() {
            self.counter_delta.set(self.counter_delta.get() - 5);
        }
        if reset.clicked() {
            self.reset_counter.set(true);
        }

        // Edge-anchored footer
        RectangleElement.place(style, "footer_bar");
        TextElement::new("WASD/Arrows: Move Cube | P: Pause Overlay | F5: Live Reload Styles").place(style, "footer_text");
    }
}

#[derive(Default)]
struct PauseOverlayScene {
    resume_requested: Cell<bool>,
    reset_pos_requested: Cell<bool>,
}

impl Scene<AppState> for PauseOverlayScene {
    fn update(&mut self, state: &mut AppState, input: &Input, _style: &Style, _dt: f32) -> SceneAction {
        if self.reset_pos_requested.take() {
            state.cube_pos = Vector3::zero();
        }

        if input.is_action_pressed("pause") || self.resume_requested.take() {
            return SceneAction::Pop;
        }
        SceneAction::None
    }

    fn draw_2d(&mut self, _state: &AppState, style: &Style) {
        // Semi-transparent backdrop overlay
        draw_rectangle_rec(get_virtual_screen_rect(), get_color(0x000000AA));

        TextElement::new("GAME PAUSED").place(style, "title");
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
            self.reset_pos_requested.set(true);
            self.resume_requested.set(true);
        }
    }
}

fn main() {
    App::init(800, 600, "MUX 0.2.0 - 3D Architecture Showcase", AppState::default)
        .set_style_file("./data/test.gss")
        .set_fps(60)
        // Configure input action and axis mapping
        .register_action("pause", [KEY_P, KEY_ESCAPE])
        .register_axis("horizontal", KEY_D, KEY_A)
        .register_axis("vertical", KEY_S, KEY_W)
        // Register scenes
        .add_scene("main", MainScene::default())
        .add_scene("pause", PauseOverlayScene::default())
        .set_active_scene("main")
        .run();
}
