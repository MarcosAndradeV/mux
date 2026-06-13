use std::path::{Path, PathBuf};

use gss::{Gss, load_gss_from_file};

pub use raylib::*;

// raylib helpers commit it back later!
use std::ffi::CString;
fn load_texture(path: CString) -> Texture {
    unsafe { LoadTexture(path.as_ptr()) }
}
fn get_color(hex: u32) -> Color {
    unsafe { GetColor(hex) }
}
fn load_music_stream(path: CString) -> Music {
    unsafe { LoadMusicStream(path.as_ptr()) }
}
fn init_audio_device() {
    unsafe {
        InitAudioDevice();
    }
}
fn close_audio_device() {
    unsafe {
        CloseAudioDevice();
    }
}

pub type Style = Gss;

struct App2<Context> {
    before_loop: Box<dyn Fn() + 'static>,
    on_update: Box<dyn Fn(&mut Context) + 'static>,
    drawing_mode: Box<dyn Fn(&Context, &Style) + 'static>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    context: Context,
}

pub struct App<Context> {
    width: i32,
    height: i32,
    title: String,
    before_loop: Option<Box<dyn Fn() + 'static>>,
    on_update: Option<Box<dyn Fn(&mut Context) + 'static>>,
    drawing_mode: Option<Box<dyn Fn(&Context, &Style) + 'static>>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    init_context: Box<dyn FnOnce() -> Context + 'static>,
}

impl<Context> AppBuilder<Context> {
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
            before_loop: None,
            on_update: None,
            drawing_mode: None,
            style_file: None,
            fps: 60,
            audio_device: false,
            init_context: Box::new(init_context),
        }
    }

    pub fn set_audio_device(mut self) -> Self {
        self.audio_device = true;
        self
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

    fn build(self) -> App<Context> {
        let Self {
            width,
            height,
            title,
            before_loop,
            on_update,
            drawing_mode,
            style_file,
            fps,
            audio_device,
            init_context,
        } = self;
        init_window(width, height, cstr!(&title));
        if audio_device {
            init_audio_device();
        }

        App {
            before_loop: before_loop.unwrap_or(Box::new(|| {})),
            on_update: on_update.unwrap_or(Box::new(|_| {})),
            drawing_mode: drawing_mode.unwrap_or(Box::new(|_, _| {})),
            style_file,
            fps,
            audio_device,
            context: init_context(),
        }
    }
}

impl<Context> AppBuilder<Context> {
    pub fn run(self) {
        let App {
            before_loop,
            on_update,
            drawing_mode,
            style_file,
            fps,
            audio_device,
            mut context,
        } = self.build();
        set_target_fps(fps);
        let mut style = load_style(style_file.as_ref(), Gss::new());
        before_loop();
        while !window_should_close() {
            if is_key_pressed(KEY_F5) {
                style = load_style(style_file.as_ref(), style);
            }
            on_update(&mut context);
            begin_drawing();
            drawing_mode(&context, &style);
            end_drawing();
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

fn get_color_field(style: &Style, path: &[&str]) -> Color {
    if let Some(string) = style.get::<String>(path) {
        map_color(string)
    } else if let Some(hex) = style.get::<u32>(path) {
        get_color(*hex)
    } else {
        BLANK
    }
}

fn get_f32_field(style: &Style, name: &str, field: &str, default: f32) -> f32 {
    if let Some(&val) = style.get::<f32>(&[name, field]) {
        val
    } else if let Some(&val) = style.get::<u32>(&[name, field]) {
        val as f32
    } else {
        default
    }
}

fn get_position_field(
    style: &Style,
    name: &str,
    field: &str,
    screen_dim: f32,
    default: f32,
) -> f32 {
    if let Some(&val) = style.get::<f32>(&[name, field]) {
        val * screen_dim
    } else if let Some(&val) = style.get::<u32>(&[name, field]) {
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
    fn place(&self, style: &Gss, name: &str) {
        let factor = get_f32_field(style, name, "factor", 1.0);

        // 1. Resolve raw coordinates (support both % and absolute pixels)
        let mut x = get_position_field(style, name, "left", get_screen_width() as f32, 0.0);
        let mut y = get_position_field(style, name, "top", get_screen_height() as f32, 0.0);

        // 2. Measure the element bounds
        let size = self.measure(factor);

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
                "middle" | "center" => y -= size.y / 2.0,
                "top" => {}
                "bottom" => y -= size.y,
                _ => {} // default to top-alignment
            }
        }

        let color = get_color_field(style, &[name, "color"]);
        self.draw(Vector2 { x, y }, factor, color);
    }
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

pub struct TextureElement(Texture2D);

impl TextureElement {
    pub fn load_texture(path: impl AsRef<str>) -> Option<Self> {
        let texture = load_texture(cstr!(path.as_ref()));
        unsafe {
            if IsTextureValid(texture) {
                Some(Self(texture))
            } else {
                None
            }
        }
    }
}

impl Drop for TextureElement {
    fn drop(&mut self) {
        unsafe {
            UnloadTexture(self.0);
        }
    }
}

impl Element for TextureElement {
    fn draw(&self, position: Vector2, factor: f32, color: Color) {
        unsafe { DrawTextureEx(self.0, position, 0.0, factor, color) };
    }

    fn measure(&self, factor: f32) -> Vector2 {
        Vector2 {
            x: self.0.width as f32 * factor,
            y: self.0.height as f32 * factor,
        }
    }
}

pub struct MusicResource(Music);

impl MusicResource {
    pub fn load_music_stream(path: impl AsRef<str>) -> Option<Self> {
        let music = load_music_stream(cstr!(path.as_ref()));
        unsafe {
            if IsMusicValid(music) {
                Some(Self(music))
            } else {
                None
            }
        }
    }
    pub fn set_volume(&self, volume: f32) {
        unsafe { SetMusicVolume(self.0, volume) };
    }
    pub fn play(&self) {
        unsafe {
            PlayMusicStream(self.0);
        }
    }
    pub fn update(&self) {
        unsafe {
            UpdateMusicStream(self.0);
        }
    }

    pub fn pause(&self) {
        unsafe {
            PauseMusicStream(self.0);
        }
    }

    pub fn resume(&self) {
        unsafe {
            ResumeMusicStream(self.0);
        }
    }
}

impl Drop for MusicResource {
    fn drop(&mut self) {
        unsafe {
            UnloadMusicStream(self.0);
        }
    }
}
