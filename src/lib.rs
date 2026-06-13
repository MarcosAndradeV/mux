use std::path::{Path, PathBuf};

use gss::{Gss, load_gss_from_file};
use raylib::*;

pub type Style = Gss;

pub struct App<Context> {
    before_loop: Option<Box<dyn Fn() + 'static>>,
    on_update: Option<Box<dyn Fn(&mut Context) + 'static>>,
    drawing_mode: Option<Box<dyn Fn(&Context, &Style) + 'static>>,
    style_file: Option<PathBuf>,
    fps: i32,
    context: Context,
}

impl<Context> App<Context> {
    pub fn init(width: i32, height: i32, title: &str, context: Context) -> App<Context> {
        init_window(width, height, cstr!(title));
        App::new(context)
    }

    fn new(context: Context) -> Self {
        Self {
            before_loop: None,
            on_update: None,
            drawing_mode: None,
            style_file: None,
            fps: 60,
            context,
        }
    }

    pub fn before_loop<F: Fn() + 'static>(mut self, f: F) -> Self {
        self.before_loop = Some(Box::new(f));
        self
    }

    pub fn on_drawing_mode<F: for<'a, 'b> Fn(&'a Context, &'b Style) + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.drawing_mode = Some(Box::new(f));
        self
    }

    pub fn on_update<F: for<'a> Fn(&'a mut Context) + 'static>(mut self, f: F) -> Self {
        self.on_update = Some(Box::new(f));
        self
    }

    pub fn set_style_file(mut self, path: &str) -> Self {
        self.style_file = Some(PathBuf::from(path));
        self
    }

    pub fn set_fps(mut self, fps: i32) -> Self {
        self.fps = fps;
        self
    }
}

impl<Context> App<Context> {
    pub fn run(self) {
        let App {
            before_loop,
            on_update,
            drawing_mode,
            style_file,
            fps,
            mut context,
        } = self;
        set_target_fps(fps);
        let mut style = load_style(style_file.as_ref(), Gss::new());
        before_loop.as_ref().inspect(|f| f());
        while !window_should_close() {
            if is_key_pressed(KEY_F5) {
                style = load_style(style_file.as_ref(), style);
            }
            on_update.as_ref().inspect(|f| f(&mut context));
            begin_drawing();
            drawing_mode.as_ref().inspect(|f| f(&context, &style));
            end_drawing();
        }
        close_window();
    }
}

fn load_style<P: AsRef<Path>>(style_file: Option<P>, fallback: Gss) -> Gss {
    if let Some(style_file) = style_file.as_ref() {
        match load_gss_from_file(style_file) {
            Ok(ok) => return ok,
            Err(err) => {
                println!(
                    "Cannot load file {} because of {}",
                    style_file.as_ref().display(),
                    err
                );
            }
        }
    }
    fallback
}

pub fn place_element(style: &Gss, name: &str, element: impl Element) {
    let factor = get_f32_field(style, name, "factor", 1.0);

    // 1. Resolve raw coordinates (support both % and absolute pixels)
    let mut x = get_position_field(style, name, "left", get_screen_width() as f32, 0.0);
    let mut y = get_position_field(style, name, "top", get_screen_height() as f32, 0.0);

    // 2. Measure the element bounds
    let size = element.measure(factor);

    // 3. Apply alignment offsets
    if let Some(align) = style.get::<String>(&[name, "align"]) {
        match align.as_str() {
            "center" => x -= size.x / 2.0,
            "right" => x -= size.x,
            _ => {} // default is left
        }
    }

    if let Some(valign) = style.get::<String>(&[name, "valign"]) {
        match valign.as_str() {
            "top" => {}
            "bottom" => y -= size.y,
            _ => y -= size.y / 2.0,// default is center
        }
    }

    let color = get_color_field(style, &[name, "color"]);
    element.draw(Vector2 { x, y }, factor, color);
}

fn get_color_field(style: &Style, path: &[&str]) -> Color {
    if let Some(string) = style.get::<String>(path) {
        map_color(string)
    } else if let Some(hex) = style.get::<i32>(path) {
        get_color(*hex)
    } else {
        BLANK
    }
}

fn get_color(hex: i32) -> Color {
    unsafe { GetColor(hex as u32) }
}

fn get_f32_field(style: &Style, name: &str, field: &str, default: f32) -> f32 {
    if let Some(&val) = style.get::<f32>(&[name, field]) {
        val
    } else if let Some(&val) = style.get::<i32>(&[name, field]) {
        val as f32
    } else {
        default
    }
}

fn get_position_field(style: &Style, name: &str, field: &str, screen_dim: f32, default: f32) -> f32 {
    if let Some(&val) = style.get::<f32>(&[name, field]) {
        val * screen_dim
    } else if let Some(&val) = style.get::<i32>(&[name, field]) {
        val as f32
    } else {
        default
    }
}

fn map_color(string: &str) -> Color {
    match string {
        "light gray" => LIGHTGRAY,
        "gray" => GRAY,
        "dark gray" => DARKGRAY,
        "yellow" => YELLOW,
        "gold" => GOLD,
        "orange" => ORANGE,
        "pink" => PINK,
        "red" => RED,
        "maroon" => MAROON,
        "green" => GREEN,
        "lime" => LIME,
        "dark green" => DARKGREEN,
        "sky blue" => SKYBLUE,
        "blue" => BLUE,
        "dark blue" => DARKBLUE,
        "purple" => PURPLE,
        "violet" => VIOLET,
        "dark purple" => DARKPURPLE,
        "beige" => BEIGE,
        "brown" => BROWN,
        "dark brown" => DARKBROWN,
        "white" => WHITE,
        "black" => BLACK,
        "magenta" => MAGENTA,
        _ => BLANK,
    }
}

pub trait Element {
    fn draw(&self, position: Vector2, factor: f32, color: Color);
    fn measure(&self, factor: f32) -> Vector2;
}

pub struct TextElement {
    pub text: String,
}

impl TextElement {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Element for TextElement {
    fn draw(&self, position: Vector2, factor: f32, color: Color) {
        draw_text(
            cstr!(&self.text),
            position.x as _,
            position.y as _,
            factor as _,
            color,
        );
    }

    fn measure(&self, factor: f32) -> Vector2 {
        Vector2 {
            x: measure_text(cstr!(&self.text), factor as _) as f32,
            y: factor,
        }
    }
}
