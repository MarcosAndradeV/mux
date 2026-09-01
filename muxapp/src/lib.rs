#![warn(missing_docs)]
//! # muxapp
//!
//! A lightweight library for building interactive applications using Raylib and Graph Style Sheets (GSS).
//!
//! - [`App`]: The main container driving the application window and render loop.
//!
//! ## Basic Example
//!
//! ```no_run
//! use muxapp::*;
//! use muxapp::muxui::*;
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

use muxutils::gss::{Gss, load_gss_from_file};
use notify::Watcher;

pub use muxui;

use muxui::*;

struct Manager<Context> {
    update: Box<dyn Fn(&mut Context, &Style) + 'static>,
    load: Box<dyn Fn(&mut Context, &Style) + 'static>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    context: Context,
    virtual_width: i32,
    virtual_height: i32,
    responsive: bool,
}

/// The main application runner that initializes the Raylib window, sets up event loops,
/// handles style reloading (both on F5 and automatic filesystem modification watch), and processes frame updates.
///
/// # Type Parameters
///
/// * `Context` - The application-defined context type passed to callbacks.
pub struct App<Context> {
    width: i32,
    height: i32,
    virtual_width: i32,
    virtual_height: i32,
    responsive: bool,
    title: String,
    update: Option<Box<dyn Fn(&mut Context, &Style) + 'static>>,
    load: Option<Box<dyn Fn(&mut Context, &Style) + 'static>>,
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
    /// * `init_context` - A closure that generates the initial application-defined context.
    pub fn init<F: FnOnce() -> Context + 'static>(
        width: i32,
        height: i32,
        title: impl Into<String>,
        init_context: F,
    ) -> Self {
        Self {
            width,
            height,
            virtual_width: width,
            virtual_height: height,
            responsive: false,
            title: title.into(),
            update: None,
            load: None,
            style_file: None,
            fps: muxutils::DEFAULT_FPS,
            audio_device: false,
            init_context: Box::new(init_context),
        }
    }
}

impl<Context> App<Context> {
    /// Enables or disables dynamic responsive screen resizing.
    ///
    /// When responsive mode is enabled (`true`), the internal virtual render canvas dynamically resizes
    /// to match the physical window resolution whenever the window is resized.
    ///
    /// When responsive mode is disabled (`false`, the default), the virtual resolution remains fixed,
    /// and window resizing maintains the aspect ratio with letterboxing/pillarboxing.
    ///
    /// # Arguments
    ///
    /// * `responsive` - `true` to enable dynamic screen sizing, `false` for fixed letterboxed virtual resolution.
    pub fn set_responsive(mut self, responsive: bool) -> Self {
        self.responsive = responsive;
        self
    }

    /// Sets the fixed internal virtual resolution for rendering offscreen textures.
    ///
    /// # Arguments
    ///
    /// * `width` - The virtual canvas width in pixels.
    /// * `height` - The virtual canvas height in pixels.
    pub fn set_virtual_resolution(mut self, width: i32, height: i32) -> Self {
        self.virtual_width = width;
        self.virtual_height = height;
        self
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
    pub fn on_update<F: Fn(&mut Context, &Style) + 'static>(mut self, f: F) -> Self {
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
    pub fn on_load<F: Fn(&mut Context, &Style) + 'static>(mut self, f: F) -> Self {
        self.load = Some(Box::new(f));
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
            virtual_width,
            virtual_height,
            responsive,
            title,
            update,
            load,
            style_file,
            fps,
            audio_device,
            init_context,
        } = self;
        log_info!(
            "MUX: Initializing window: {}x{} (Virtual: {}x{}) - \"{}\"",
            width,
            height,
            virtual_width,
            virtual_height,
            title
        );
        set_config_flags(&[FLAG_WINDOW_RESIZABLE]);
        init_window(width, height, cstr!(&title));
        if audio_device {
            log_info!("MUX: Initializing audio device");
            init_audio_device();
        }

        Manager {
            update: update.unwrap_or(Box::new(|_, _| {})),
            load: load.unwrap_or(Box::new(|_, _| {})),
            style_file,
            fps,
            audio_device,
            context: init_context(),
            virtual_width,
            virtual_height,
            responsive,
        }
    }

    /// Runs the main application loop.
    ///
    /// This method builds the application manager, initializes the Raylib window,
    /// sets up the filesystem modification watcher if a style file was specified,
    /// and executes the update loop until the window is requested to close.
    pub fn run(self) {
        log_info!("MUX: Starting App run loop");
        let Manager {
            update,
            load,
            style_file,
            fps,
            audio_device,
            mut context,
            mut virtual_width,
            mut virtual_height,
            responsive,
        } = self.build();
        set_target_fps(fps);

        let mut target = load_render_texture(virtual_width, virtual_height);

        let (tx, rx) = std::sync::mpsc::channel();
        let mut style = load_style_fallback(style_file.as_ref(), Style::new());

        load(&mut context, &style);

        // 1. Declare the watcher OUTSIDE the block so it lives longer
        let mut _watcher = None;

        if let Some(ref path) = style_file {
            let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());

            if let Some(parent) = abs_path.parent() {
                let abs_path_clone = abs_path.clone();
                let tx_clone = tx.clone();

                if let Ok(mut w) =
                    notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                        if let Ok(event) = res
                            && event.paths.iter().any(|p| p == &abs_path_clone)
                            && (event.kind.is_modify() || event.kind.is_create())
                        {
                            let _ = tx_clone.send(());
                        }
                    })
                {
                    if w.watch(parent, notify::RecursiveMode::NonRecursive).is_ok() {
                        log_info!(
                            "MUX: File watcher set up for style file: {}",
                            abs_path.display()
                        );
                        // 2. Assign it here
                        _watcher = Some(w);
                    } else {
                        log_warn!(
                            "MUX: Failed to watch directory for style file: {}",
                            parent.display()
                        );
                    }
                }
            }
        }

        while !window_should_close() {
            let window_w = get_screen_width();
            let window_h = get_screen_height();

            if responsive {
                let target_w = window_w.max(1);
                let target_h = window_h.max(1);
                if target_w != virtual_width || target_h != virtual_height {
                    virtual_width = target_w;
                    virtual_height = target_h;
                    if is_render_texture_valid(target) {
                        unload_render_texture(target);
                    }
                    target = load_render_texture(virtual_width, virtual_height);
                    log_info!(
                        "MUX: Window resized, updated virtual resolution to {}x{}",
                        virtual_width,
                        virtual_height
                    );
                }
            }

            muxutils::set_viewport(
                virtual_width as f32,
                virtual_height as f32,
                window_w as f32,
                window_h as f32,
            );

            if is_key_pressed(KEY_F5) {
                log_info!("MUX: F5 pressed. Reloading style...");
                style = load_style_fallback(style_file.as_ref(), style);
            }

            let mut should_reload = false;
            while rx.try_recv().is_ok() {
                should_reload = true;
            }
            if should_reload {
                log_info!("MUX: Style file modified. Reloading style...");
                style = load_style_fallback(style_file.as_ref(), style);
                load(&mut context, &style);
            }

            begin_texture_mode(target);
            clear_background(get_color(0x181818FF));

            update(&mut context, &style);
            end_texture_mode();

            begin_drawing();
            clear_background(BLACK);
            let (src, dest) = muxutils::get_viewport_rects();
            draw_texture_pro(target.texture, src, dest, Vector2::zero(), 0.0, WHITE);
            end_drawing();
        }

        log_info!("MUX: Window close requested. Cleaning up...");

        if is_render_texture_valid(target) {
            unload_render_texture(target);
        }

        drop(context);
        muxui::clear_texture_cache();
        if audio_device {
            log_info!("MUX: Closing audio device");
            close_audio_device();
        }
        close_window();
        log_info!("MUX: App terminated");
    }
}

fn load_style_fallback<P: AsRef<Path>>(style_file: Option<P>, fallback: Gss) -> Gss {
    if let Some(style_file) = style_file.as_ref() {
        match load_gss_from_file(style_file) {
            Ok(ok) => {
                log_info!(
                    "MUX: Style loaded successfully from {}",
                    style_file.as_ref().display()
                );
                return ok;
            }
            Err(err) => {
                log_warn!(
                    "MUX: Cannot load file {} because of {}",
                    style_file.as_ref().display(),
                    err
                );
            }
        }
    }
    fallback
}
