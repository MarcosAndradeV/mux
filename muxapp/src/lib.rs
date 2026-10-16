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
//!
//! fn main() {
//!     App::new(800, 600, "My App")
//!         .on_update(|ctx, style| {
//!             clear_background(get_color(0x181818FF));
//!             TextElement::new("Hello, Muxui!").place(style, "title");
//!         })
//!         .run();
//! }
//! ```

use std::path::{Path, PathBuf};

use muxutils::gss::{Gss, load_gss_from_file};
use notify::Watcher;

pub use muxui;

use muxui::*;

pub mod input;
pub mod scene;

pub use input::*;
pub use scene::*;

type UpdateCallback<Context> = Box<dyn Fn(&mut Context, &Style) + 'static>;
type UpdateWithInputCallback<Context> = Box<dyn Fn(&mut Context, &Input, &Style, f32) + 'static>;
type LoadCallback<Context> = Box<dyn Fn(&mut Context, &Style) + 'static>;

struct Manager<Context> {
    update: UpdateCallback<Context>,
    update_with_input: Option<UpdateWithInputCallback<Context>>,
    scenes: SceneRouter<Context>,
    initial_scene: Option<String>,
    input: Input,
    load: LoadCallback<Context>,
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
    update: Option<UpdateCallback<Context>>,
    update_with_input: Option<UpdateWithInputCallback<Context>>,
    scenes: SceneRouter<Context>,
    initial_scene: Option<String>,
    input: Input,
    load: Option<LoadCallback<Context>>,
    style_file: Option<PathBuf>,
    fps: i32,
    audio_device: bool,
    exit_key: Option<Option<KeyboardKey>>,
    cursor_mode: CursorMode,
    init_context: Box<dyn FnOnce() -> Context + 'static>,
}

impl App<()> {
    /// Initializes a new [`App`] instance with the specified window dimensions and title,
    /// with an empty unit context `()`.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the application window in pixels.
    /// * `height` - The height of the application window in pixels.
    /// * `title` - The title of the application window.
    pub fn new(width: i32, height: i32, title: impl Into<String>) -> Self {
        Self::init(width, height, title, || ())
    }
}

impl<Context> App<Context> {
    /// Initializes a new [`App`] instance with the specified window dimensions, title, and initial context.
    ///
    /// Responsive dynamic resizing is enabled (`true`) by default.
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
            responsive: true,
            title: title.into(),
            update: None,
            update_with_input: None,
            scenes: SceneRouter::new(),
            initial_scene: None,
            input: Input::new(),
            load: None,
            style_file: None,
            fps: muxutils::DEFAULT_FPS,
            audio_device: false,
            exit_key: None,
            cursor_mode: CursorMode::Normal,
            init_context: Box::new(init_context),
        }
    }
}

impl<Context> App<Context> {
    /// Enables or disables dynamic responsive screen resizing.
    ///
    /// When responsive mode is enabled (`true`, the default), the internal virtual render canvas dynamically resizes
    /// to match the physical window resolution whenever the window is resized.
    ///
    /// When responsive mode is disabled (`false`), the virtual resolution remains fixed,
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

    /// Safely configures the key that closes the application window (passing `None` disables `KEY_ESCAPE` exit).
    pub fn set_exit_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.exit_key = Some(key);
        self
    }

    /// Safely disables the default `KEY_ESCAPE` window-close shortcut so `KEY_ESCAPE` can be used for pause menus.
    pub fn disable_exit_key(mut self) -> Self {
        self.exit_key = Some(None);
        self
    }

    /// Configures the initial mouse cursor mode (`Normal`, `Hidden`, or `Captured`).
    pub fn set_cursor_mode(mut self, mode: CursorMode) -> Self {
        self.cursor_mode = mode;
        self
    }

    /// Registers a scene with a unique identifier.
    ///
    /// If no active scene has been set yet, this scene becomes the initial active scene.
    pub fn add_scene(mut self, name: impl Into<String>, scene: impl Scene<Context> + 'static) -> Self {
        let name_str = name.into();
        if self.initial_scene.is_none() {
            self.initial_scene = Some(name_str.clone());
        }
        self.scenes.register(name_str, scene);
        self
    }

    /// Sets the initial active scene by its registered name.
    pub fn set_active_scene(mut self, name: impl Into<String>) -> Self {
        self.initial_scene = Some(name.into());
        self
    }

    /// Sets an update callback that receives access to the [`Input`] mapper and frame delta time (`dt`).
    pub fn on_update_with_input<F: Fn(&mut Context, &Input, &Style, f32) + 'static>(mut self, f: F) -> Self {
        self.update_with_input = Some(Box::new(f));
        self
    }

    /// Registers a named action mapped to one or more keyboard keys.
    pub fn register_action(mut self, name: impl Into<String>, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.input.register_action(name, keys);
        self
    }

    /// Registers a named UI/system action mapped to keyboard keys that remains active even when gameplay input is masked.
    pub fn register_ui_action(mut self, name: impl Into<String>, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.input.register_ui_action(name, keys);
        self
    }

    /// Registers a named action mapped to one or more mouse buttons.
    pub fn register_mouse_action(mut self, name: impl Into<String>, buttons: impl IntoIterator<Item = MouseButton>) -> Self {
        self.input.register_mouse_action(name, buttons);
        self
    }

    /// Registers a 1D virtual axis mapped to a positive key (+1.0) and a negative key (-1.0).
    pub fn register_axis(mut self, name: impl Into<String>, positive: KeyboardKey, negative: KeyboardKey) -> Self {
        self.input.register_axis(name, positive, negative);
        self
    }

    /// Registers a continuous mouse delta axis with a sensitivity multiplier.
    pub fn register_mouse_axis(mut self, name: impl Into<String>, axis: MouseAxis, sensitivity: f32) -> Self {
        self.input.register_mouse_axis(name, axis, sensitivity);
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

    /// Sets the callback function to run when the stylesheet is loaded or reloaded.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that accepts the mutable application context and style context.
    pub fn on_style_reload<F: Fn(&mut Context, &Style) + 'static>(mut self, f: F) -> Self {
        self.load = Some(Box::new(f));
        self
    }

    /// Sets the callback function to run when the style file is reloaded.
    ///
    /// Compatibility alias for [`on_style_reload`].
    pub fn on_load<F: Fn(&mut Context, &Style) + 'static>(self, f: F) -> Self {
        self.on_style_reload(f)
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
            update_with_input,
            scenes,
            initial_scene,
            input,
            load,
            style_file,
            fps,
            audio_device,
            exit_key,
            cursor_mode,
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
        if let Some(key_opt) = exit_key {
            muxutils::set_exit_key(key_opt);
        }
        muxutils::set_cursor_mode(cursor_mode);
        if audio_device {
            log_info!("MUX: Initializing audio device");
            init_audio_device();
        }

        Manager {
            update: update.unwrap_or(Box::new(|_, _| {})),
            update_with_input,
            scenes,
            initial_scene,
            input,
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
            update_with_input,
            mut scenes,
            initial_scene,
            input,
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

        if let Some(ref init_scene) = initial_scene {
            scenes.switch(init_scene, &mut context, &style);
        }

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

            let mut should_reload = false;
            if is_key_pressed(KEY_F5) {
                log_info!("MUX: F5 pressed. Reloading style...");
                should_reload = true;
            }

            while rx.try_recv().is_ok() {
                should_reload = true;
            }
            if should_reload {
                log_info!("MUX: Style reloaded.");
                style = load_style_fallback(style_file.as_ref(), style);
                load(&mut context, &style);
            }

            let dt = get_frame_time();

            begin_texture_mode(target);
            clear_background(get_color(0x181818FF));

            if !scenes.step(&mut context, &input, &style, dt) {
                if let Some(ref update_input) = update_with_input {
                    update_input(&mut context, &input, &style, dt);
                } else {
                    update(&mut context, &style);
                }
            }
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

        if let Some(scene) = scenes.active_scene_mut() {
            scene.on_exit(&mut context);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_init_defaults() {
        let app = App::init(800, 600, "Default App", || 100);
        assert_eq!(app.width, 800);
        assert_eq!(app.height, 600);
        assert_eq!(app.virtual_width, 800);
        assert_eq!(app.virtual_height, 600);
        assert!(app.responsive);
        assert_eq!(app.title, "Default App");
        assert_eq!(app.fps, muxutils::DEFAULT_FPS);
        assert!(!app.audio_device);
        assert!(app.style_file.is_none());
        assert!(app.update.is_none());
        assert!(app.load.is_none());
        assert_eq!((app.init_context)(), 100);
    }

    #[test]
    fn test_app_new() {
        let app = App::new(800, 600, "Simple App");
        assert_eq!(app.width, 800);
        assert_eq!(app.height, 600);
        assert!(app.responsive);
        assert_eq!((app.init_context)(), ());
    }

    #[test]
    fn test_app_builder_methods() {
        let app = App::init(640, 480, "Configured App", || String::from("context"))
            .set_responsive(false)
            .set_virtual_resolution(1920, 1080)
            .set_audio_device()
            .disable_exit_key()
            .set_cursor_mode(CursorMode::Captured)
            .set_fps(60)
            .set_style_file("custom.gss")
            .register_action("jump", [KEY_SPACE])
            .register_ui_action("pause", [KEY_ESCAPE])
            .register_axis("pitch", KEY_W, KEY_S)
            .on_update(|ctx, _style| {
                ctx.push_str("_updated");
            })
            .on_update_with_input(|ctx, _input, _style, _dt| {
                ctx.push_str("_input");
            })
            .on_style_reload(|ctx, _style| {
                ctx.push_str("_reloaded");
            });

        assert_eq!(app.width, 640);
        assert_eq!(app.height, 480);
        assert_eq!(app.virtual_width, 1920);
        assert_eq!(app.virtual_height, 1080);
        assert!(!app.responsive);
        assert_eq!(app.title, "Configured App");
        assert_eq!(app.fps, 60);
        assert!(app.audio_device);
        assert_eq!(app.exit_key, Some(None));
        assert_eq!(app.cursor_mode, CursorMode::Captured);
        assert_eq!(app.style_file, Some(PathBuf::from("custom.gss")));
        assert!(app.update.is_some());
        assert!(app.update_with_input.is_some());
        assert!(app.load.is_some());
        assert_eq!((app.init_context)(), "context");
    }

    struct DummyScene;
    impl Scene<()> for DummyScene {}

    #[test]
    fn test_app_scene_registration() {
        let app = App::new(800, 600, "Scene App")
            .add_scene("intro", DummyScene)
            .add_scene("main", DummyScene)
            .set_active_scene("main");

        assert_eq!(app.initial_scene, Some("main".to_string()));
        assert!(app.scenes.has_scene("intro"));
        assert!(app.scenes.has_scene("main"));
    }

    #[test]
    fn test_load_style_fallback_none() {
        let fallback = Gss::new();
        let loaded = load_style_fallback::<&str>(None, fallback);
        assert_eq!(loaded.get::<u32>(&["anything"]), None);
    }

    #[test]
    fn test_load_style_fallback_missing_file() {
        let fallback = muxutils::gss::parse_str("key = 42,").unwrap();
        let loaded = load_style_fallback(Some("non_existent_file_12345.gss"), fallback);
        assert_eq!(loaded.get::<u32>(&["key"]), Some(&42));
    }

    #[test]
    fn test_load_style_fallback_valid_file() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_mux_valid_{}.gss", std::process::id()));
        std::fs::write(&file_path, "button = { width = 120, height = 40 },").unwrap();

        let fallback = Gss::new();
        let loaded = load_style_fallback(Some(&file_path), fallback);
        assert_eq!(loaded.get::<u32>(&["button", "width"]), Some(&120));
        assert_eq!(loaded.get::<u32>(&["button", "height"]), Some(&40));

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_load_style_fallback_invalid_syntax() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_mux_invalid_{}.gss", std::process::id()));
        std::fs::write(&file_path, "INVALID GSS SYNTAX %%% {{").unwrap();

        let fallback = muxutils::gss::parse_str("fallback_val = 99,").unwrap();
        let loaded = load_style_fallback(Some(&file_path), fallback);
        assert_eq!(loaded.get::<u32>(&["fallback_val"]), Some(&99));

        let _ = std::fs::remove_file(file_path);
    }
}
