// #![warn(missing_docs)]
//! # muxui
//!
//! A lightweight user interface library built on top of Raylib (`raylib-rs`) and styled
//! using Graph Style Sheets (GSS).
//!
//! ## Overview
//!
//! `muxui` provides a simple, declarative way to lay out and render graphical elements in a window.
//! The library's core structure revolves around:
//! - [`App`]: The main container driving the application window and render loop.
//! - [`Element`]: A trait representing any UI widget that can be measured, drawn, and handle events.
//! - Standard elements: [`TextElement`], [`TextureElement`], [`RectangleElemet`], [`ButtonElement`], and [`StackLayout`].
//! - Styling via [`Style`] (alias for [`Gss`]), which resolves layout configurations like margins, spacing, and colors.
//!
//! ## Basic Example
//!
//! ```no_run
//! use muxui::*;
//!
//! struct AppContext;
//!
//! fn main() {
//!     App::init(800, 600, "My App", || AppContext)
//!         .on_update(|ctx, style| {
//!             begin_drawing();
//!             clear_background(get_color(0x181818FF));
//!             TextElement::new("Hello, Muxui!").place(style, "title");
//!             end_drawing();
//!         })
//!         .run();
//! }
//! ```

use std::path::{Path, PathBuf};

use gss::{Gss, load_gss_from_file};
use notify::Watcher;

pub use raylib::*;

// raylib helpers commit it back later!

const DEFAULT_ROTATION: f32 = 0.0;
const DEFAULT_FONT_SIZE: f32 = 20.0;
const DEFAULT_SPACING: f32 = 2.0;
const DEFAULT_SCALE: f32 = 1.0;
const DEFAULT_FPS: i32 = 24;

const DEBUG_FRAME_LINE_THICK: f32 = 2.0;
const MOUSE_CLICK_RADIUS: f32 = 2.0;

/// Type alias for the Graph Style Sheets ([`Gss`]) context used to style UI components.
pub type Style = Gss;

struct Manager<Context> {
    update: Box<dyn Fn(&mut Context, &Style) + 'static>,
    reload: Box<dyn Fn(&mut Context, &Style) + 'static>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    context: Context,
}

/// The main application runner that initializes the Raylib window, sets up event loops,
/// handles style reloading (both on F5 and automatic filesystem modification watch), and processes frame updates.
///
/// # Type Parameters
///
/// * `Context` - The application-defined state/context type passed to callbacks.
pub struct App<Context> {
    width: i32,
    height: i32,
    title: String,
    update: Option<Box<dyn Fn(&mut Context, &Style) + 'static>>,
    reload: Option<Box<dyn Fn(&mut Context, &Style) + 'static>>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    init_context: Box<dyn FnOnce() -> Context + 'static>,
}

impl<Context> App<Context> {
    /// Initializes a new [`App`] instance with the specified window dimensions, title, and initial context.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the application window in pixels.
    /// * `height` - The height of the application window in pixels.
    /// * `title` - The title of the application window.
    /// * `init_context` - A closure that generates the initial application-defined state/context.
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
            reload: None,
            style_file: None,
            fps: DEFAULT_FPS,
            audio_device: false,
            init_context: Box::new(init_context),
        }
    }

    /// Enables the audio device for the application.
    ///
    /// If called, the audio device is initialized when the application starts running, and closed on cleanup.
    pub fn set_audio_device(mut self) -> Self {
        self.audio_device = true;
        self
    }

    /// Sets the callback function to run on every frame update.
    ///
    /// The update function is called on every frame and is responsible for processing events,
    /// updating the state, and drawing elements to the screen.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that accepts the mutable application context and style context.
    pub fn on_update<F: for<'a, 'b> Fn(&'a mut Context, &Style) + 'static>(mut self, f: F) -> Self {
        self.update = Some(Box::new(f));
        self
    }

    /// Sets the callback function to run when the style file is reloaded.
    ///
    /// The callback receives the mutable context and the newly loaded style.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that accepts the mutable application context and style context.
    pub fn on_reload<F: for<'a, 'b> Fn(&'a mut Context, &Style) + 'static>(mut self, f: F) -> Self {
        self.reload = Some(Box::new(f));
        self
    }

    /// Sets the file path for the style sheet configuration.
    ///
    /// If configured, the app will watch this file for modifications and reload it automatically
    /// at runtime. The file can also be reloaded manually by pressing the F5 key.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to the style sheet (e.g. "style.gss").
    pub fn set_style_file(mut self, path: &str) -> Self {
        self.style_file = Some(PathBuf::from(path));
        self
    }

    /// Sets the target frames per second (FPS) for the update loop.
    ///
    /// # Arguments
    ///
    /// * `fps` - The target frame rate (e.g. 60).
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
            reload,
            style_file,
            fps,
            audio_device,
            init_context,
        } = self;
        log_info!(
            "MUXUI: Initializing window: {}x{} - \"{}\"",
            width,
            height,
            title
        );
        unsafe { SetConfigFlags(FLAG_WINDOW_RESIZABLE as u32) };
        init_window(width, height, cstr!(&title));
        if audio_device {
            log_info!("MUXUI: Initializing audio device");
            init_audio_device();
        }

        Manager {
            update: update.unwrap_or(Box::new(|_, _| {
                begin_drawing();
                end_drawing();
            })),
            reload: reload.unwrap_or(Box::new(|_, _| {})),
            style_file,
            fps,
            audio_device,
            context: init_context(),
        }
    }
}

impl<Context> App<Context> {
    /// Runs the main application loop.
    ///
    /// This method builds the application manager, initializes the Raylib window,
    /// sets up the filesystem modification watcher if a style file was specified,
    /// and executes the update loop until the window is requested to close.
    pub fn run(self) {
        log_info!("MUXUI: Starting App run loop");
        let Manager {
            update,
            reload,
            style_file,
            fps,
            audio_device,
            mut context,
        } = self.build();
        set_target_fps(fps);

        let (tx, rx) = std::sync::mpsc::channel();
        let mut style = load_style_fallback(style_file.as_ref(), Style::new());
        reload(&mut context, &style);

        // 1. Declare the watcher OUTSIDE the block so it lives longer
        let mut _watcher = None;

        if let Some(ref path) = style_file {
            let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());

            if let Some(parent) = abs_path.parent() {
                let abs_path_clone = abs_path.clone();
                let tx_clone = tx.clone();

                if let Ok(mut w) =
                    notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                        if let Ok(event) = res {
                            if event.paths.iter().any(|p| p == &abs_path_clone) {
                                if event.kind.is_modify() || event.kind.is_create() {
                                    let _ = tx_clone.send(());
                                }
                            }
                        }
                    })
                {
                    if w.watch(parent, notify::RecursiveMode::NonRecursive).is_ok() {
                        log_info!(
                            "MUXUI: File watcher set up for style file: {}",
                            abs_path.display()
                        );
                        // 2. Assign it here
                        _watcher = Some(w);
                    } else {
                        log_warn!(
                            "MUXUI: Failed to watch directory for style file: {}",
                            parent.display()
                        );
                    }
                }
            }
        }

        while !window_should_close() {
            if is_key_pressed(KEY_F5) {
                log_info!("MUXUI: F5 pressed. Reloading style...");
                style = load_style_fallback(style_file.as_ref(), style);
            }

            let mut should_reload = false;
            while rx.try_recv().is_ok() {
                should_reload = true;
            }
            if should_reload {
                log_info!("MUXUI: Style file modified. Reloading style...");
                style = load_style_fallback(style_file.as_ref(), style);
                reload(&mut context, &style);
            }

            update(&mut context, &style);
        }
        log_info!("MUXUI: Window close requested. Cleaning up...");
        drop(context);
        if audio_device {
            log_info!("MUXUI: Closing audio device");
            close_audio_device();
        }
        close_window();
        log_info!("MUXUI: App terminated");
    }
}

fn load_style_fallback<P: AsRef<Path>>(style_file: Option<P>, fallback: Gss) -> Gss {
    if let Some(style_file) = style_file.as_ref() {
        match load_gss_from_file(style_file) {
            Ok(ok) => {
                log_info!(
                    "MUXUI: Style loaded successfully from {}",
                    style_file.as_ref().display()
                );
                return ok;
            }
            Err(err) => {
                log_warn!(
                    "MUXUI: Cannot load file {} because of {}",
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

pub fn get_relative_field(style: &Style, path: &[&str], scale: f32, default: f32) -> f32 {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    None,
    ButtonClicked,
}

pub trait Element {
    /// Draw the raw element on the screen at a resolved position
    fn draw(&self, position: Vector2, style: &Style, name: &str);
    /// Get the element size
    fn measure(&self, style: &Style, name: &str) -> Vector2;

    fn event(&self) -> Event {
        Event::None
    }

    /// Place the element on the screen
    fn place(&self, style: &Style, name: &str) {
        let position = self.get_position(style, name);
        self.draw(position, style, name);
        if get_bool_field(style, name, "frame", false) {
            draw_rectangle_lines_ex(self.get_rec(style, name), DEBUG_FRAME_LINE_THICK, GREEN);
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
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
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

impl std::ops::Deref for TextureElement {
    type Target = Texture2D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Texture> for TextureElement {
    fn from(value: Texture) -> Self {
        Self(value)
    }
}

impl TextureElement {
    pub fn load_from_file(path: impl AsRef<str>) -> Option<Self> {
        let path_str = path.as_ref();
        let texture = load_texture(cstr!(path_str));
        if is_texture_valid(texture) {
            log_info!("MUXUI: Successfully loaded texture from \"{}\"", path_str);
            Some(Self(texture))
        } else {
            log_error!("MUXUI: Failed to load texture from \"{}\"", path_str);
            None
        }
    }
    pub fn invalid() -> Self {
        Self(Texture::default())
    }
}

impl Drop for TextureElement {
    fn drop(&mut self) {
        if is_texture_valid(self.0) {
            unload_texture(self.0);
        }
    }
}

impl Element for TextureElement {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        if is_texture_valid(self.0) {
            let scale = get_f32_field(style, &[name, "scale"], DEFAULT_SCALE);
            let rotation = get_f32_field(style, &[name, "rotation"], DEFAULT_ROTATION);
            let color = get_color_field(style, &[name, "color"], WHITE);
            draw_texture_ex(self.0, position, rotation, scale, color);
        }
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
    cached_rec: std::cell::Cell<Rectangle>,
}

impl<E: Element> ButtonElement<E> {
    pub fn new(element: E) -> Self {
        Self {
            element,
            cached_rec: std::cell::Cell::new(Rectangle {
                x: -1000.0,
                y: -1000.0,
                width: 0.0,
                height: 0.0,
            }),
        }
    }

    fn click(&self) -> bool {
        is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
            && check_collision_circle_rec(
                get_mouse_position(),
                MOUSE_CLICK_RADIUS,
                self.cached_rec.get(),
            )
    }

    pub fn element(&self) -> &E {
        &self.element
    }

    pub fn element_mut(&mut self) -> &mut E {
        &mut self.element
    }
}

impl<E: Element> Element for ButtonElement<E> {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        let color = get_color_field(style, &[name, "button", "color"], BLANK);
        let size = self.measure(style, name);
        let rec = Rectangle {
            x: position.x,
            y: position.y,
            width: size.x,
            height: size.y,
        };
        self.cached_rec.set(rec);
        unsafe {
            DrawRectangleRec(rec, color);
        }
        self.element.draw(position, style, name);
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        self.element.measure(style, name)
    }

    fn event(&self) -> Event {
        if self.click() {
            Event::ButtonClicked
        } else {
            Event::None
        }
    }
}

pub struct StackLayout<'a, 'b> {
    children: &'b [(&'a str, &'a dyn Element)],
}

impl<'a, 'b> StackLayout<'a, 'b> {
    pub fn new(children: &'b [(&'a str, &'a dyn Element)]) -> Self {
        Self { children }
    }

    pub fn children(&self) -> &[(&'a str, &'a dyn Element)] {
        &self.children
    }
}

impl<'a, 'b> Element for StackLayout<'a, 'b> {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        let direction = style
            .get::<String>(&[name, "direction"])
            .cloned()
            .unwrap_or_else(|| "vertical".to_string());
        let gap = get_f32_field(style, &[name, "gap"], 0.0);
        let stack_size = self.measure(style, name);

        let mut offset = 0.0;
        for (child_name, child) in self.children {
            let child_size = child.measure(style, child_name);

            let child_pos = match direction.as_str() {
                "horizontal" => {
                    let mut child_y = position.y;
                    if let Some(valign) = style.get::<String>(&[child_name, "valign"]) {
                        match valign.as_str() {
                            "middle" | "center" => child_y += (stack_size.y - child_size.y) / 2.0,
                            "bottom" => child_y += stack_size.y - child_size.y,
                            _ => {}
                        }
                    }
                    let pos = Vector2 {
                        x: position.x + offset,
                        y: child_y,
                    };
                    offset += child_size.x + gap;
                    pos
                }
                _ => {
                    // vertical
                    let mut child_x = position.x;
                    if let Some(align) = style.get::<String>(&[child_name, "align"]) {
                        match align.as_str() {
                            "center" => child_x += (stack_size.x - child_size.x) / 2.0,
                            "right" => child_x += stack_size.x - child_size.x,
                            _ => {}
                        }
                    }
                    let pos = Vector2 {
                        x: child_x,
                        y: position.y + offset,
                    };
                    offset += child_size.y + gap;
                    pos
                }
            };

            child.draw(child_pos, style, child_name);

            if get_bool_field(style, child_name, "frame", false) {
                let rec = Rectangle {
                    x: child_pos.x,
                    y: child_pos.y,
                    width: child_size.x,
                    height: child_size.y,
                };
                draw_rectangle_lines_ex(rec, DEBUG_FRAME_LINE_THICK, GREEN);
            }
        }
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        let direction = style
            .get::<String>(&[name, "direction"])
            .cloned()
            .unwrap_or_else(|| "vertical".to_string());
        let gap = get_f32_field(style, &[name, "gap"], 0.0);

        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        let mut count = 0;

        for (child_name, child) in self.children {
            let child_size = child.measure(style, child_name);
            match direction.as_str() {
                "horizontal" => {
                    width += child_size.x;
                    height = height.max(child_size.y);
                }
                _ => {
                    // vertical
                    width = width.max(child_size.x);
                    height += child_size.y;
                }
            }
            count += 1;
        }

        if count > 1 {
            match direction.as_str() {
                "horizontal" => width += gap * (count - 1) as f32,
                _ => height += gap * (count - 1) as f32,
            }
        }

        Vector2 {
            x: width,
            y: height,
        }
    }
}

pub struct RectangleElemet;

impl Element for RectangleElemet {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        unsafe {
            DrawRectangleV(
                position,
                self.measure(style, name),
                get_color_field(style, &[name, "color"], MAGENTA),
            )
        };
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        Vector2::new(
            style.get_or_default(&[name, "width"]),
            style.get_or_default(&[name, "height"]),
        )
    }
}

type UpdateFn<E, Payload> = fn(&mut E, Payload);

pub struct UpdateElemet<Payload, E: Element>(E, UpdateFn<E, Payload>);

impl<P, E: Element> UpdateElemet<P, E> {
    pub fn new(element: E, f: UpdateFn<E, P>) -> Self {
        Self(element, f)
    }

    pub fn update(&mut self, payload: P) {
        (self.1)(&mut self.0, payload);
    }
}

impl<P, E: Element> Element for UpdateElemet<P, E> {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        self.0.draw(position, style, name);
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        self.0.measure(style, name)
    }
}
