pub use gss;
pub use raylib;

pub const DEFAULT_ROTATION: f32 = 0.0;
pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_SPACING: f32 = 2.0;
pub const DEFAULT_SCALE: f32 = 1.0;
pub const DEBUG_FRAME_LINE_THICK: f32 = 2.0;
pub const DEFAULT_FPS: i32 = 24;

/// Raylib trace logging function wrapper.
pub fn trace_log(level: raylib::TraceLogLevel, message: &str) {
    let c_msg = std::ffi::CString::new(message).unwrap_or_default();
    unsafe {
        raylib::TraceLog(level as std::os::raw::c_int, c"%s".as_ptr(), c_msg.as_ptr());
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::trace_log($crate::raylib::LOG_INFO, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::trace_log($crate::raylib::LOG_WARNING, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::trace_log($crate::raylib::LOG_ERROR, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::trace_log($crate::raylib::LOG_DEBUG, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::trace_log($crate::raylib::LOG_TRACE, &format!($($arg)*))
    };
}

/// Represents the virtual viewport scaling configuration for aspect-preserved resolution.
#[derive(Debug, Clone, Copy)]
pub struct ViewportState {
    /// Virtual canvas width.
    pub virtual_width: f32,
    /// Virtual canvas height.
    pub virtual_height: f32,
    /// Actual window width.
    pub window_width: f32,
    /// Actual window height.
    pub window_height: f32,
    /// Viewport scaling ratio.
    pub scale: f32,
    /// Horizontal offset for letterboxing.
    pub offset_x: f32,
    /// Vertical offset for letterboxing.
    pub offset_y: f32,
    /// Scaled viewport width inside window.
    pub viewport_width: f32,
    /// Scaled viewport height inside window.
    pub viewport_height: f32,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            virtual_width: 800.0,
            virtual_height: 600.0,
            window_width: 800.0,
            window_height: 600.0,
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            viewport_width: 800.0,
            viewport_height: 600.0,
        }
    }
}

static VIEWPORT: std::sync::RwLock<ViewportState> = std::sync::RwLock::new(ViewportState {
    virtual_width: 800.0,
    virtual_height: 600.0,
    window_width: 800.0,
    window_height: 600.0,
    scale: 1.0,
    offset_x: 0.0,
    offset_y: 0.0,
    viewport_width: 800.0,
    viewport_height: 600.0,
});

/// Sets the current virtual viewport state and calculates letterbox scaling offsets.
pub fn set_viewport(virtual_w: f32, virtual_h: f32, window_w: f32, window_h: f32) {
    let scale = (window_w / virtual_w).min(window_h / virtual_h);
    let viewport_width = virtual_w * scale;
    let viewport_height = virtual_h * scale;
    let offset_x = (window_w - viewport_width) / 2.0;
    let offset_y = (window_h - viewport_height) / 2.0;

    if let Ok(mut vp) = VIEWPORT.write() {
        *vp = ViewportState {
            virtual_width: virtual_w,
            virtual_height: virtual_h,
            window_width: window_w,
            window_height: window_h,
            scale,
            offset_x,
            offset_y,
            viewport_width,
            viewport_height,
        };
    }
}

/// Returns a copy of the current [`ViewportState`].
pub fn get_viewport_state() -> ViewportState {
    VIEWPORT.read().map(|vp| *vp).unwrap_or_default()
}

/// Retrieves the virtual screen width.
pub fn get_virtual_screen_width() -> f32 {
    get_viewport_state().virtual_width
}

/// Retrieves the virtual screen height.
pub fn get_virtual_screen_height() -> f32 {
    get_viewport_state().virtual_height
}

/// Retrieves the virtual screen dimensions as a [`raylib::Vector2`].
pub fn get_virtual_screen_size() -> raylib::Vector2 {
    let vp = get_viewport_state();
    raylib::Vector2::new(vp.virtual_width, vp.virtual_height)
}

/// Computes the transformed mouse position in virtual canvas coordinates.
pub fn get_virtual_mouse_position() -> raylib::Vector2 {
    let vp = get_viewport_state();
    let raw_mouse = raylib::get_mouse_position();
    let x = (raw_mouse.x - vp.offset_x) / vp.scale;
    let y = (raw_mouse.y - vp.offset_y) / vp.scale;
    raylib::Vector2::new(x, y)
}

/// Computes the source (OpenGL inverted Y) and destination (window letterbox) rectangles.
pub fn get_viewport_rects() -> (raylib::Rectangle, raylib::Rectangle) {
    let vp = get_viewport_state();
    let src = raylib::Rectangle::new(0.0, 0.0, vp.virtual_width, -vp.virtual_height);
    let dest = raylib::Rectangle::new(
        vp.offset_x,
        vp.offset_y,
        vp.viewport_width,
        vp.viewport_height,
    );
    (src, dest)
}

/// Retrieves the virtual screen bounding rectangle as a [`raylib::Rectangle`].
pub fn get_virtual_screen_rect() -> raylib::Rectangle {
    let vp = get_viewport_state();
    raylib::Rectangle::new(0.0, 0.0, vp.virtual_width, vp.virtual_height)
}

/// Checks if a given rectangle intersects the visible virtual screen bounds.
pub fn is_rect_on_screen(rec: raylib::Rectangle) -> bool {
    let screen = get_virtual_screen_rect();
    if rec.width <= 0.0 || rec.height <= 0.0 {
        rec.x >= 0.0 && rec.x <= screen.width && rec.y >= 0.0 && rec.y <= screen.height
    } else {
        check_collision_recs(rec, screen)
    }
}

pub fn map_color(string: &str) -> raylib::Color {
    let normalized = string.to_lowercase().replace(['_', '-'], " ");
    match normalized.trim() {
        "light gray" | "lightgray" => raylib::LIGHTGRAY,
        "gray" | "grey" => raylib::GRAY,
        "dark gray" | "darkgray" | "dark grey" | "darkgrey" => raylib::DARKGRAY,
        "yellow" => raylib::YELLOW,
        "gold" => raylib::GOLD,
        "orange" => raylib::ORANGE,
        "pink" => raylib::PINK,
        "red" => raylib::RED,
        "maroon" => raylib::MAROON,
        "green" => raylib::GREEN,
        "lime" => raylib::LIME,
        "dark green" | "darkgreen" => raylib::DARKGREEN,
        "sky blue" | "skyblue" => raylib::SKYBLUE,
        "blue" => raylib::BLUE,
        "dark blue" | "darkblue" => raylib::DARKBLUE,
        "purple" => raylib::PURPLE,
        "violet" => raylib::VIOLET,
        "dark purple" | "darkpurple" => raylib::DARKPURPLE,
        "beige" => raylib::BEIGE,
        "brown" => raylib::BROWN,
        "dark brown" | "darkbrown" => raylib::DARKBROWN,
        "white" => raylib::WHITE,
        "black" => raylib::BLACK,
        "magenta" => raylib::MAGENTA,
        "blank" | "transparent" => raylib::BLANK,
        _ => raylib::BLANK,
    }
}

fn normalize_path<'a>(path: &[&'a str]) -> Vec<&'a str> {
    if path.iter().any(|p| p.contains('.')) {
        path.iter().flat_map(|p| p.split('.')).collect()
    } else {
        path.to_vec()
    }
}

/// Retrieves a color property from the stylesheet [`Style`] at the given path segment slice (e.g., `&["button", "color"]`).
///
/// It supports reading color values defined as string names (e.g. `"red"`, `"dark green"`),
/// hex integers (e.g. `0xFF00FFFF`), or references. If not found or invalid, `default` is returned.
pub fn get_color_field(obj: &gss::Object, path: &[&str], default: raylib::Color) -> raylib::Color {
    let norm = normalize_path(path);
    if let Some(string) = obj.get::<String>(&norm) {
        let mapped = map_color(string);
        if mapped != raylib::BLANK
            || string.eq_ignore_ascii_case("blank")
            || string.eq_ignore_ascii_case("transparent")
        {
            return mapped;
        }
    }
    if let Some(&hex) = obj.get::<u32>(&norm) {
        raylib::get_color(hex)
    } else {
        default
    }
}

/// Retrieves a boolean property from the stylesheet [`Style`] for a specific element name and field.
///
/// Returns `default` if the field is not present.
pub fn get_bool_field(obj: &gss::Object, name: &str, field: &str, default: bool) -> bool {
    let norm = normalize_path(&[name, field]);
    obj.get::<bool>(&norm).copied().unwrap_or(default)
}

/// Retrieves a float (`f32`) property from the stylesheet [`Style`] at the given path.
///
/// Automatically handles float literals, percentage values (e.g. `89%` -> `0.89`), and integers (`u32`).
/// Returns `default` if the field is not present or cannot be parsed as a float.
pub fn get_f32_field(obj: &gss::Object, path: &[&str], default: f32) -> f32 {
    let norm = normalize_path(path);
    if let Some(&val) = obj.get::<f32>(&norm) {
        val
    } else if let Some(&val) = obj.get::<u32>(&norm) {
        val as f32
    } else if let Some(&val) = obj.get::<i32>(&norm) {
        val as f32
    } else {
        default
    }
}

/// Retrieves a relative coordinate or float property from the stylesheet [`Style`] at the given path.
///
/// - If the retrieved value is a percentage (e.g. `80%`) or float (e.g. `0.8`), it is scaled by `scale`.
/// - If the retrieved value is an exact integer literal (e.g. `120`), it is treated as absolute pixels.
///
/// Returns `default` if the field is not present.
pub fn get_relative_field(obj: &gss::Object, path: &[&str], scale: f32, default: f32) -> f32 {
    get_optional_relative_field(obj, path, scale).unwrap_or(default)
}

/// Retrieves an optional relative coordinate or float property from the stylesheet [`Style`].
///
/// - If the retrieved value is a percentage (e.g. `80%`) or float (e.g. `0.8`), it is scaled by `scale`.
/// - If the retrieved value is an exact integer literal (e.g. `120`), it is treated as absolute pixels.
///
/// Returns `None` if the field is not present in the style.
pub fn get_optional_relative_field(obj: &gss::Object, path: &[&str], scale: f32) -> Option<f32> {
    let norm = normalize_path(path);
    if let Some(&val) = obj.get::<u32>(&norm) {
        Some(val as f32)
    } else if let Some(&val) = obj.get::<i32>(&norm) {
        Some(val as f32)
    } else if let Some(&val) = obj.get::<f32>(&norm) {
        Some(val * scale)
    } else {
        None
    }
}

/// Retrieves a string property from the stylesheet [`Style`] at the given path.
///
/// Returns `default` if the field is not present.
pub fn get_string_field(gss: &gss::Object, path: &[&str], default: &str) -> String {
    let norm = normalize_path(path);
    gss.get::<String>(&norm)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

/// Retrieves a `usize` property from the stylesheet [`Style`] at the given path.
///
/// Automatically converts integer literals and floats into `usize`.
/// Returns `default` if the field is not present or cannot be parsed.
pub fn get_usize_field(obj: &gss::Object, path: &[&str], default: usize) -> usize {
    let norm = normalize_path(path);
    if let Some(&val) = obj.get::<usize>(&norm) {
        val
    } else if let Some(&val) = obj.get::<u32>(&norm) {
        val as usize
    } else if let Some(&val) = obj.get::<i32>(&norm) {
        val.max(0) as usize
    } else if let Some(&val) = obj.get::<f32>(&norm) {
        val.max(0.0) as usize
    } else {
        default
    }
}

/// Returns true if the window has been initialized successfully.
pub fn is_window_ready() -> bool {
    unsafe { raylib::IsWindowReady() }
}

/// Returns true if the render texture is valid (GPU framebuffer allocated).
pub fn is_render_texture_valid(target: raylib::RenderTexture2D) -> bool {
    unsafe { raylib::IsRenderTextureValid(target) }
}

/// Unloads render texture from GPU memory (framebuffer).
pub fn unload_render_texture(target: raylib::RenderTexture2D) {
    unsafe {
        raylib::UnloadRenderTexture(target);
    }
}

/// Loads a render texture (framebuffer) with the given dimensions.
pub fn load_render_texture(width: i32, height: i32) -> raylib::RenderTexture2D {
    unsafe { raylib::LoadRenderTexture(width, height) }
}

/// Initializes render texture for drawing.
pub fn begin_texture_mode(target: raylib::RenderTexture2D) {
    unsafe {
        raylib::BeginTextureMode(target);
    }
}

/// Ends drawing to render texture.
pub fn end_texture_mode() {
    unsafe {
        raylib::EndTextureMode();
    }
}

/// Draws a part of a texture (defined by a rectangle) with 'pro' parameters.
pub fn draw_texture_pro(
    texture: raylib::Texture2D,
    source: raylib::Rectangle,
    dest: raylib::Rectangle,
    origin: raylib::Vector2,
    rotation: f32,
    tint: raylib::Color,
) {
    unsafe {
        raylib::DrawTexturePro(texture, source, dest, origin, rotation, tint);
    }
}

/// Draws a rectangle with a `Rectangle` struct.
pub fn draw_rectangle_rec(rec: raylib::Rectangle, color: raylib::Color) {
    unsafe {
        raylib::DrawRectangleRec(rec, color);
    }
}

/// Draws a rectangle using Vector2 position and size.
pub fn draw_rectangle_v(position: raylib::Vector2, size: raylib::Vector2, color: raylib::Color) {
    unsafe {
        raylib::DrawRectangleV(position, size, color);
    }
}

/// Checks collision between two rectangles.
pub fn check_collision_recs(rec1: raylib::Rectangle, rec2: raylib::Rectangle) -> bool {
    unsafe { raylib::CheckCollisionRecs(rec1, rec2) }
}

/// Checks collision between point and rectangle.
pub fn check_collision_point_rec(point: raylib::Vector2, rec: raylib::Rectangle) -> bool {
    unsafe { raylib::CheckCollisionPointRec(point, rec) }
}

/// Draws a circle with integer center and float radius.
pub fn draw_circle(center_x: i32, center_y: i32, radius: f32, color: raylib::Color) {
    unsafe {
        raylib::DrawCircle(center_x, center_y, radius, color);
    }
}

/// Loads a texture from an Image.
pub fn load_texture_from_image(image: raylib::Image) -> raylib::Texture2D {
    unsafe { raylib::LoadTextureFromImage(image) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_calculation() {
        // Virtual 800x600, Window 1600x1200 -> Scale 2.0, Offset (0, 0)
        set_viewport(800.0, 600.0, 1600.0, 1200.0);
        let vp = get_viewport_state();
        assert_eq!(vp.scale, 2.0);
        assert_eq!(vp.offset_x, 0.0);
        assert_eq!(vp.offset_y, 0.0);
        assert_eq!(vp.viewport_width, 1600.0);
        assert_eq!(vp.viewport_height, 1200.0);

        // Virtual 800x600, Window 1920x1080 -> Scale 1.8, Viewport 1440x1080, Offset_x 240
        set_viewport(800.0, 600.0, 1920.0, 1080.0);
        let vp = get_viewport_state();
        assert_eq!(vp.scale, 1.8);
        assert_eq!(vp.offset_x, 240.0);
        assert_eq!(vp.offset_y, 0.0);
        assert_eq!(vp.viewport_width, 1440.0);
        assert_eq!(vp.viewport_height, 1080.0);

        let (src, dest) = get_viewport_rects();
        assert_eq!(src.width, 800.0);
        assert_eq!(src.height, -600.0);
        assert_eq!(dest.x, 240.0);
        assert_eq!(dest.y, 0.0);
        assert_eq!(dest.width, 1440.0);
        assert_eq!(dest.height, 1080.0);
    }

    #[test]
    fn test_gss_helpers() {
        let style = gss::parse_str(
            r#"
            base_col = "gold",
            btn = {
                color = base_col,
                hex_color = 0xFF00FFFF,
                scale = 50%,
                pixel_offset = 150,
                width = 300.5,
                count = 4,
                active = true,
                title = "Fireball",
            },
            "#,
        )
        .unwrap();

        // Color resolution (symbol reference and hex)
        assert_eq!(
            get_color_field(&style, &["btn", "color"], raylib::WHITE),
            raylib::GOLD
        );
        assert_eq!(
            get_color_field(&style, &["btn", "hex_color"], raylib::WHITE),
            raylib::get_color(0xFF00FFFF)
        );
        assert_eq!(
            get_color_field(&style, &["btn", "missing"], raylib::RED),
            raylib::RED
        );

        // Boolean resolution
        assert!(get_bool_field(&style, "btn", "active", false));
        assert!(!get_bool_field(&style, "btn", "missing", false));

        // Float resolution (exact float and converted int)
        assert_eq!(get_f32_field(&style, &["btn", "width"], 0.0), 300.5);
        assert_eq!(get_f32_field(&style, &["btn", "count"], 0.0), 4.0);

        // Relative resolution (Percentage 50% * 800.0 = 400.0, Absolute pixels = 150.0)
        assert_eq!(
            get_relative_field(&style, &["btn", "scale"], 800.0, 0.0),
            400.0
        );
        assert_eq!(
            get_relative_field(&style, &["btn", "pixel_offset"], 800.0, 0.0),
            150.0
        );
        assert_eq!(
            get_optional_relative_field(&style, &["btn", "scale"], 800.0),
            Some(400.0)
        );
        assert_eq!(
            get_optional_relative_field(&style, &["btn", "pixel_offset"], 800.0),
            Some(150.0)
        );
        assert_eq!(
            get_optional_relative_field(&style, &["btn", "missing"], 800.0),
            None
        );

        // String and usize resolution
        assert_eq!(
            get_string_field(&style, &["btn", "title"], "Default"),
            "Fireball"
        );
        assert_eq!(get_usize_field(&style, &["btn", "count"], 1), 4);
    }

    #[test]
    fn test_responsive_viewport_and_culling() {
        set_viewport(1280.0, 720.0, 1280.0, 720.0);
        let vp = get_viewport_state();
        assert_eq!(vp.scale, 1.0);
        assert_eq!(vp.offset_x, 0.0);
        assert_eq!(vp.offset_y, 0.0);
        assert_eq!(vp.viewport_width, 1280.0);
        assert_eq!(vp.viewport_height, 720.0);

        let screen_rec = get_virtual_screen_rect();
        assert_eq!(screen_rec.width, 1280.0);
        assert_eq!(screen_rec.height, 720.0);

        // Within screen
        assert!(is_rect_on_screen(raylib::Rectangle::new(
            100.0, 100.0, 50.0, 50.0
        )));
        // Partially intersecting screen
        assert!(is_rect_on_screen(raylib::Rectangle::new(
            -20.0, 100.0, 50.0, 50.0
        )));
        // Outside screen (right)
        assert!(!is_rect_on_screen(raylib::Rectangle::new(
            5000.0, 100.0, 50.0, 50.0
        )));
        // Outside screen (above)
        assert!(!is_rect_on_screen(raylib::Rectangle::new(
            100.0, -100.0, 50.0, 50.0
        )));
        // Point/zero-sized rect inside screen
        assert!(is_rect_on_screen(raylib::Rectangle::new(
            500.0, 500.0, 0.0, 0.0
        )));
        // Point/zero-sized rect outside screen
        assert!(!is_rect_on_screen(raylib::Rectangle::new(
            2000.0, 500.0, 0.0, 0.0
        )));
    }

    #[test]
    fn test_map_color() {
        assert_eq!(map_color("light gray"), raylib::LIGHTGRAY);
        assert_eq!(map_color("lightgray"), raylib::LIGHTGRAY);
        assert_eq!(map_color("light_gray"), raylib::LIGHTGRAY);
        assert_eq!(map_color("gray"), raylib::GRAY);
        assert_eq!(map_color("grey"), raylib::GRAY);
        assert_eq!(map_color("dark gray"), raylib::DARKGRAY);
        assert_eq!(map_color("darkgray"), raylib::DARKGRAY);
        assert_eq!(map_color("dark-grey"), raylib::DARKGRAY);
        assert_eq!(map_color("yellow"), raylib::YELLOW);
        assert_eq!(map_color("gold"), raylib::GOLD);
        assert_eq!(map_color("orange"), raylib::ORANGE);
        assert_eq!(map_color("pink"), raylib::PINK);
        assert_eq!(map_color("red"), raylib::RED);
        assert_eq!(map_color("RED"), raylib::RED);
        assert_eq!(map_color("maroon"), raylib::MAROON);
        assert_eq!(map_color("green"), raylib::GREEN);
        assert_eq!(map_color("lime"), raylib::LIME);
        assert_eq!(map_color("dark green"), raylib::DARKGREEN);
        assert_eq!(map_color("dark-green"), raylib::DARKGREEN);
        assert_eq!(map_color("sky blue"), raylib::SKYBLUE);
        assert_eq!(map_color("skyblue"), raylib::SKYBLUE);
        assert_eq!(map_color("sky_blue"), raylib::SKYBLUE);
        assert_eq!(map_color("blue"), raylib::BLUE);
        assert_eq!(map_color("dark blue"), raylib::DARKBLUE);
        assert_eq!(map_color("dark_blue"), raylib::DARKBLUE);
        assert_eq!(map_color("purple"), raylib::PURPLE);
        assert_eq!(map_color("violet"), raylib::VIOLET);
        assert_eq!(map_color("dark purple"), raylib::DARKPURPLE);
        assert_eq!(map_color("dark-purple"), raylib::DARKPURPLE);
        assert_eq!(map_color("beige"), raylib::BEIGE);
        assert_eq!(map_color("brown"), raylib::BROWN);
        assert_eq!(map_color("dark brown"), raylib::DARKBROWN);
        assert_eq!(map_color("dark_brown"), raylib::DARKBROWN);
        assert_eq!(map_color("white"), raylib::WHITE);
        assert_eq!(map_color("black"), raylib::BLACK);
        assert_eq!(map_color("magenta"), raylib::MAGENTA);
        assert_eq!(map_color("blank"), raylib::BLANK);
        assert_eq!(map_color("transparent"), raylib::BLANK);
        assert_eq!(map_color("invalid_color_xyz"), raylib::BLANK);
    }

    #[test]
    fn test_get_color_field_edge_cases() {
        let style = gss::parse_str(
            r#"
            colors = {
                str_blank = "blank",
                str_transparent = "transparent",
                invalid_str = "not_a_color",
                hex = 0x12345678,
            },
            "#,
        )
        .unwrap();

        assert_eq!(
            get_color_field(&style, &["colors", "str_blank"], raylib::WHITE),
            raylib::BLANK
        );
        assert_eq!(
            get_color_field(&style, &["colors", "str_transparent"], raylib::WHITE),
            raylib::BLANK
        );
        // Invalid color string falls back to default
        assert_eq!(
            get_color_field(&style, &["colors", "invalid_str"], raylib::RED),
            raylib::RED
        );
        // Hex u32 color
        assert_eq!(
            get_color_field(&style, &["colors", "hex"], raylib::WHITE),
            raylib::get_color(0x12345678)
        );
        // Missing path falls back to default
        assert_eq!(
            get_color_field(&style, &["colors", "nonexistent"], raylib::BLUE),
            raylib::BLUE
        );
    }

    #[test]
    fn test_numeric_field_conversions_and_defaults() {
        let style = gss::parse_str(
            r#"
            vals = {
                pos_f32 = 3.5,
                pos_u32 = 100,
                zero_val = 0,
                flag_t = true,
                flag_f = false,
                name = "hello",
            },
            "#,
        )
        .unwrap();

        // get_f32_field
        assert_eq!(get_f32_field(&style, &["vals", "pos_f32"], 0.0), 3.5);
        assert_eq!(get_f32_field(&style, &["vals", "pos_u32"], 0.0), 100.0);
        assert_eq!(get_f32_field(&style, &["vals", "zero_val"], 9.0), 0.0);
        assert_eq!(get_f32_field(&style, &["vals", "missing"], 9.0), 9.0);

        // get_relative_field
        assert_eq!(
            get_relative_field(&style, &["vals", "pos_u32"], 800.0, 0.0),
            100.0
        );
        assert_eq!(
            get_relative_field(&style, &["vals", "pos_f32"], 100.0, 0.0),
            350.0
        );
        assert_eq!(
            get_relative_field(&style, &["vals", "zero_val"], 100.0, 50.0),
            0.0
        );
        assert_eq!(
            get_relative_field(&style, &["vals", "missing"], 800.0, 50.0),
            50.0
        );

        // get_usize_field
        assert_eq!(get_usize_field(&style, &["vals", "pos_u32"], 0), 100);
        assert_eq!(get_usize_field(&style, &["vals", "pos_f32"], 0), 3);
        assert_eq!(get_usize_field(&style, &["vals", "zero_val"], 99), 0);
        assert_eq!(get_usize_field(&style, &["vals", "missing"], 7), 7);

        // get_string_field
        assert_eq!(
            get_string_field(&style, &["vals", "name"], "default"),
            "hello"
        );
        assert_eq!(
            get_string_field(&style, &["vals", "missing"], "fallback"),
            "fallback"
        );

        // get_bool_field
        assert!(get_bool_field(&style, "vals", "flag_t", false));
        assert!(!get_bool_field(&style, "vals", "flag_f", true));
        assert!(get_bool_field(&style, "vals", "missing", true));
        assert!(!get_bool_field(&style, "vals", "missing", false));
    }

    #[test]
    fn test_viewport_state_getters_and_defaults() {
        let def = ViewportState::default();
        assert_eq!(def.virtual_width, 800.0);
        assert_eq!(def.virtual_height, 600.0);
        assert_eq!(def.window_width, 800.0);
        assert_eq!(def.window_height, 600.0);
        assert_eq!(def.scale, 1.0);
        assert_eq!(def.offset_x, 0.0);
        assert_eq!(def.offset_y, 0.0);

        set_viewport(1920.0, 1080.0, 1920.0, 1080.0);
        assert_eq!(get_virtual_screen_width(), 1920.0);
        assert_eq!(get_virtual_screen_height(), 1080.0);

        let size = get_virtual_screen_size();
        assert_eq!(size.x, 1920.0);
        assert_eq!(size.y, 1080.0);

        let rect = get_virtual_screen_rect();
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 1920.0);
        assert_eq!(rect.height, 1080.0);
    }

    #[test]
    fn test_logging_macros_no_panic() {
        log_info!("Test info log: {}", 123);
        log_warn!("Test warn log: {}", "warning message");
        log_error!("Test error log: {}", "error message");
        log_debug!("Test debug log: {}", 456);
        log_trace!("Test trace log: {}", 789);
    }
}
