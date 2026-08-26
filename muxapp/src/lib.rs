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

use muxengine::{EngineController, GameState};
use muxutils::gss::{Gss, load_gss_from_file, Object};
use notify::Watcher;

pub use muxengine;
pub use muxui;

use muxui::*;

/// The application context wrapping both the user-defined state and the point-and-click engine.
///
/// It acts as the primary interface for callbacks to inspect or mutate both
/// the custom application state and the active game engine controller.
pub struct Context<State> {
    state: State,
    engine: EngineController,
}

impl<State> Context<State> {
    /// Returns a shared reference to the user-defined application state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Returns a shared reference to the underlying engine controller.
    pub fn engine(&self) -> &EngineController {
        &self.engine
    }

    /// Returns a mutable reference to the underlying engine controller.
    pub fn engine_mut(&mut self) -> &mut EngineController {
        &mut self.engine
    }

    /// Returns a mutable reference to the user-defined application state.
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

struct Manager<AppState> {
    update: Box<dyn Fn(&mut Context<AppState>, &Style) + 'static>,
    load: Box<dyn Fn(&mut Context<AppState>, &Style) + 'static>,
    script_hook: Box<dyn Fn(&mut GameState, &str) -> Option<String> + 'static>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    context: Context<AppState>,
    virtual_width: i32,
    virtual_height: i32,
}

/// The main application runner that initializes the Raylib window, sets up event loops,
/// handles style reloading (both on F5 and automatic filesystem modification watch), and processes frame updates.
///
/// # Type Parameters
///
/// * `AppState` - The application-defined state type passed to callbacks.
pub struct App<AppState> {
    width: i32,
    height: i32,
    virtual_width: i32,
    virtual_height: i32,
    title: String,
    update: Option<Box<dyn Fn(&mut Context<AppState>, &Style) + 'static>>,
    load: Option<Box<dyn Fn(&mut Context<AppState>, &Style) + 'static>>,
    script_hook: Option<Box<dyn Fn(&mut GameState, &str) -> Option<String> + 'static>>,
    style_file: Option<PathBuf>,
    initial_scene: Option<String>,
    fps: i32,
    audio_device: bool,
    init_state: Box<dyn FnOnce() -> AppState + 'static>,
}

impl<AppState> App<AppState> {
    /// Initializes a new [`App`] instance with the specified window dimensions, title, and initial context.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the application window in pixels.
    /// * `height` - The height of the application window in pixels.
    /// * `title` - The title of the application window.
    /// * `init_state` - A closure that generates the initial application-defined state.
    pub fn init<F: FnOnce() -> AppState + 'static>(
        width: i32,
        height: i32,
        title: impl Into<String>,
        init_state: F,
    ) -> Self {
        Self {
            width,
            height,
            virtual_width: width,
            virtual_height: height,
            title: title.into(),
            update: None,
            load: None,
            script_hook: None,
            style_file: None,
            initial_scene: None,
            fps: muxutils::DEFAULT_FPS,
            audio_device: false,
            init_state: Box::new(init_state),
        }
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
    pub fn on_update<F: Fn(&mut Context<AppState>, &Style) + 'static>(mut self, f: F) -> Self {
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
    pub fn on_load<F: Fn(&mut Context<AppState>, &Style) + 'static>(mut self, f: F) -> Self {
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

    /// Set an optional hook to run custom script triggers.
    pub fn set_script_hook<F>(mut self, hook: F) -> Self
    where
        F: Fn(&mut GameState, &str) -> Option<String> + 'static,
    {
        self.script_hook = Some(Box::new(hook));
        self
    }

    fn build(self) -> Manager<AppState> {
        let Self {
            width,
            height,
            virtual_width,
            virtual_height,
            title,
            update,
            load,
            style_file,
            fps,
            audio_device,
            init_state,
            initial_scene,
            script_hook,
        } = self;
        log_info!(
            "MUX: Initializing window: {}x{} (Virtual: {}x{}) - \"{}\"",
            width,
            height,
            virtual_width,
            virtual_height,
            title
        );
        unsafe { SetConfigFlags(FLAG_WINDOW_RESIZABLE as u32) };
        init_window(width, height, cstr!(&title));
        if audio_device {
            log_info!("MUX: Initializing audio device");
            init_audio_device();
        }

        Manager {
            update: update.unwrap_or(Box::new(|_, _| {})),
            load: load.unwrap_or(Box::new(|_, _| {})),
            script_hook: script_hook.unwrap_or(Box::new(|_, _| None)),
            style_file,
            fps,
            audio_device,
            context: Context {
                state: init_state(),
                engine: EngineController::new(initial_scene.unwrap_or_default()),
            },
            virtual_width,
            virtual_height,
        }
    }

    /// Sets the inital scene for the engine
    pub fn set_initial_scene(mut self, initial_scene: impl ToString) -> Self {
        self.initial_scene = Some(initial_scene.to_string());
        self
    }
}

impl<AppContext> App<AppContext> {
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
            script_hook,
            virtual_width,
            virtual_height,
        } = self.build();
        set_target_fps(fps);

        let target = load_render_texture(virtual_width, virtual_height);

        let (tx, rx) = std::sync::mpsc::channel();
        let mut style = load_style_fallback(style_file.as_ref(), Style::new());

        context.engine.set_script_hook(script_hook);
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
            let window_w = get_screen_width() as f32;
            let window_h = get_screen_height() as f32;
            muxutils::set_viewport(
                virtual_width as f32,
                virtual_height as f32,
                window_w,
                window_h,
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

            // Automatically render the active scene's background texture
            let view = context.engine.current_view(&style);
            let scene_style_path = ["scenes", &view.scene_id];
            if style.get::<Object>(&scene_style_path).is_some() {
                let bg_el = TextureElement::new(&view.background_texture);
                let bg_path = format!("scenes.{}.background", view.scene_id);
                bg_el.place(&style, &bg_path);
            } else {
                let texture = get_cached_texture(&view.background_texture);
                if is_texture_valid(texture) {
                    draw_texture_ex(texture, Vector2::zero(), 0.0, 1.0, WHITE);
                }
            }

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
