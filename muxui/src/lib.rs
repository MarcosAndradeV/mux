use std::path::{Path, PathBuf};

use gss::{Gss, load_gss_from_file};

pub use raylib::*;

// raylib helpers commit it back later!
// use std::ffi::CString;


const DEFAULT_ROTATION: f32 = 0.0;
const DEFAULT_FONT_SIZE: f32 = 20.0;
const DEFAULT_SPACING: f32 = 2.0;
const DEFAULT_SCALE: f32 = 1.0;
const DEFAULT_FPS: i32 = 60;

const DEBUG_FRAME_LINE_THICK: f32 = 2.0;
const MOUSE_CLICK_RADIUS: f32 = 2.0;

pub type Style = Gss;

struct Manager<Context> {
    update: Box<dyn Fn(&mut Context, &Style) + 'static>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    context: Context,
}

pub struct App<Context> {
    width: i32,
    height: i32,
    title: String,
    update: Option<Box<dyn Fn(&mut Context, &Style) + 'static>>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    init_context: Box<dyn FnOnce() -> Context + 'static>,
}

impl<Context> App<Context> {
    pub fn init<F: FnOnce() -> Context + 'static>(
        width: i32,
        height: i32,
        title: impl Into<String>,
        init_context: F,
    ) -> Self {
        Self {
            width,
            height,
            title: title.into(),
            update: None,
            style_file: None,
            fps: DEFAULT_FPS,
            audio_device: false,
            init_context: Box::new(init_context),
        }
    }

    pub fn set_audio_device(mut self) -> Self {
        self.audio_device = true;
        self
    }

    pub fn on_update<F: for<'a, 'b> Fn(&'a mut Context, &Style) + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.update = Some(Box::new(f));
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

    fn build(self) -> Manager<Context> {
        let Self {
            width,
            height,
            title,
            update,
            style_file,
            fps,
            audio_device,
            init_context,
        } = self;
        unsafe { SetConfigFlags(FLAG_WINDOW_RESIZABLE as u32) };
        init_window(width, height, cstr!(&title));
        if audio_device {
            init_audio_device();
        }

        Manager {
            update: update.unwrap_or(Box::new(|_, _| {})),
            style_file,
            fps,
            audio_device,
            context: init_context(),
        }
    }
}

impl<Context> App<Context> {
    pub fn run(self) {
        let Manager {
            update,
            style_file,
            fps,
            audio_device,
            mut context,
        } = self.build();
        set_target_fps(fps);
        let mut style = load_style(style_file.as_ref(), Gss::new());
        while !window_should_close() {
            if is_key_pressed(KEY_F5) {
                style = load_style(style_file.as_ref(), style);
            }
            update(&mut context, &style);
        }
        drop(context);
        if audio_device {
            close_audio_device();
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

pub fn get_color_field(style: &Style, path: &[&str], default: Color) -> Color {
    if let Some(string) = style.get::<String>(path) {
        map_color(string)
    } else if let Some(hex) = style.get::<u32>(path) {
        get_color(*hex)
    } else {
        default
    }
}

pub fn get_bool_field(style: &Style, name: &str, field: &str, default: bool) -> bool {
    if let Some(&val) = style.get::<bool>(&[name, field]) {
        val
    } else {
        default
    }
}

pub fn get_f32_field(style: &Style, path: &[&str], default: f32) -> f32 {
    if let Some(&val) = style.get::<f32>(path) {
        val
    } else if let Some(&val) = style.get::<u32>(path) {
        val as f32
    } else {
        default
    }
}

pub fn get_relative_field(
    style: &Style,
    path: &[&str],
    scale: f32,
    default: f32,
) -> f32 {
    if let Some(&val) = style.get::<f32>(path) {
        val * scale
    } else if let Some(&val) = style.get::<u32>(path) {
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
    /// Draw the raw element on the screen
    fn draw(&self, style: &Gss, name: &str);
    /// Get the element size
    fn measure(&self, style: &Style, name: &str) -> Vector2;

    /// Place the element on the screen
    fn place(&self, style: &Gss, name: &str) {
        self.draw(style, name);
        if get_bool_field(style, name, "frame", false) {
            draw_rectangle_lines_ex(self.get_rec(style, name), DEBUG_FRAME_LINE_THICK, GREEN);
            return;
        }
    }

    /// Get the element [`Rectangle`]
    fn get_rec(&self, style: &Gss, name: &str) -> Rectangle {
        let Vector2 { x, y } = self.get_position(style, name);
        let Vector2 {
            x: width,
            y: height,
        } = self.measure(style, name);
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Get the element 2d position as [`Vector2`]
    fn get_position(&self, style: &Gss, name: &str) -> Vector2 {
        let mut x = get_relative_field(style, &[name, "left"], get_screen_width() as f32, 0.0);
        let mut y = get_relative_field(style, &[name, "top"], get_screen_height() as f32, 0.0);
        let size = self.measure(style, name);

        // Apply horizontal alignment (defaults to left-aligned if absent/invalid)
        if let Some(align) = style.get::<String>(&[name, "align"]) {
            match align.as_str() {
                "center" => x -= size.x / 2.0,
                "right" => x -= size.x,
                _ => {}
            }
        }

        // Apply vertical alignment (defaults to top-aligned if absent/invalid)
        if let Some(valign) = style.get::<String>(&[name, "valign"]) {
            match valign.as_str() {
                "middle" | "center" => y -= size.y / 2.0,
                "bottom" => y -= size.y,
                _ => {}
            }
        }

        Vector2 { x, y }
    }
}

pub struct TextElement {
    pub text: String,
    font: Font,
}

impl TextElement {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font: get_font_default(),
        }
    }
    pub fn create(text: impl Into<String>, font: Font) -> Self {
        Self {
            text: text.into(),
            font,
        }
    }
}

impl Element for TextElement {
    fn draw(&self, style: &Style, name: &str) {
        let position = self.get_position(style, name);
        let font_size = get_f32_field(style, &[name, "font_size"], DEFAULT_FONT_SIZE);
        let color = get_color_field(style, &[name, "color"], WHITE);
        let spacing = get_f32_field(style, &[name, "spacing"], DEFAULT_SPACING);
        draw_text_ex(
            self.font,
            cstr!(&self.text),
            position,
            font_size,
            spacing,
            color,
        );
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        let font_size = get_f32_field(style, &[name, "font_size"], DEFAULT_FONT_SIZE);
        let spacing = get_f32_field(style, &[name, "spacing"], DEFAULT_SPACING);
        measure_text_ex(self.font, cstr!(&self.text), font_size, spacing)
    }
}

pub struct TextureElement(Texture2D);

impl TextureElement {
    pub fn load_from_file(path: impl AsRef<str>) -> Option<Self> {
        let texture = load_texture(cstr!(path.as_ref()));
        if is_texture_valid(texture) {
            Some(Self(texture))
        } else {
            None
        }
    }
}

impl Drop for TextureElement {
    fn drop(&mut self) {
        unload_texture(self.0);
    }
}

impl Element for TextureElement {
    fn draw(&self, style: &Style, name: &str) {
        let position = self.get_position(style, name);
        let scale = get_f32_field(style, &[name, "scale"], DEFAULT_SCALE);
        let rotation = get_f32_field(style, &[name, "rotation"], DEFAULT_ROTATION);
        let color = get_color_field(style, &[name, "color"], WHITE);
        draw_texture_ex(self.0, position, rotation, scale, color);
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        let scale = get_f32_field(style, &[name, "scale"], DEFAULT_SCALE);
        Vector2 {
            x: self.0.width as f32 * scale,
            y: self.0.height as f32 * scale,
        }
    }
}

pub struct ButtonElement<E: Element> {
    element: E,
}

impl<E: Element> ButtonElement<E> {
    pub fn new(element: E) -> Self {
        Self { element }
    }

    pub fn click(&self, style: &gss::Object, name: &str) -> bool {
        is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
            && check_collision_circle_rec(
                get_mouse_position(),
                MOUSE_CLICK_RADIUS,
                self.get_rec(style, name),
            )
    }
}

impl<'a, E: Element> Element for ButtonElement<E> {
    fn draw(&self, style: &Style, name: &str) {
        let color = get_color_field(style, &[name, "button", "color"], BLANK);
        let rec = self.get_rec(style, name);
        unsafe {
            DrawRectangleRec(rec, color);
        }
        self.element.draw(style, name);
    }
    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        self.element.measure(style, name)
    }
}
