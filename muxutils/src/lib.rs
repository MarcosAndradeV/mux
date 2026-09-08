pub use gss;
pub use raylib;

pub const DEFAULT_ROTATION: f32 = 0.0;
pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_SPACING: f32 = 2.0;
pub const DEFAULT_SCALE: f32 = 1.0;
pub const DEBUG_FRAME_LINE_THICK: f32 = 2.0;
pub const MOUSE_CLICK_RADIUS: f32 = 2.0;
pub const DEFAULT_FPS: i32 = 24;

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
        raylib::check_collision_recs(rec, screen)
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

/// Retrieves a color property from the stylesheet [`Style`] at the given path segment slice (e.g., `&["button", "color"]`).
///
/// It supports reading color values defined as string names (e.g. `"red"`, `"dark green"`),
/// hex integers (e.g. `0xFF00FFFF`), or references. If not found or invalid, `default` is returned.
pub fn get_color_field(obj: &gss::Object, path: &[&str], default: raylib::Color) -> raylib::Color {
    if let Some(string) = obj.get::<String>(path) {
        let mapped = map_color(string);
        if mapped != raylib::BLANK || string.eq_ignore_ascii_case("blank") {
            return mapped;
        }
    }
    if let Some(&hex) = obj.get::<u32>(path) {
        raylib::get_color(hex)
    } else {
        default
    }
}

/// Retrieves a boolean property from the stylesheet [`Style`] for a specific element name and field.
///
/// Returns `default` if the field is not present.
pub fn get_bool_field(obj: &gss::Object, name: &str, field: &str, default: bool) -> bool {
    obj.get::<bool>(&[name, field]).copied().unwrap_or(default)
}

/// Retrieves a float (`f32`) property from the stylesheet [`Style`] at the given path.
///
/// Automatically handles float literals, percentage values (e.g. `89%` -> `0.89`), and integers (`u32`).
/// Returns `default` if the field is not present or cannot be parsed as a float.
pub fn get_f32_field(obj: &gss::Object, path: &[&str], default: f32) -> f32 {
    if let Some(&val) = obj.get::<f32>(path) {
        val
    } else if let Some(&val) = obj.get::<u32>(path) {
        val as f32
    } else if let Some(&val) = obj.get::<i32>(path) {
        val as f32
    } else {
        default
    }
}

/// Retrieves a relative coordinate or float property from the stylesheet [`Style`] at the given path.
///
/// - If the retrieved value is a percentage (e.g. `80%`) or float (e.g. `0.8`), it is scaled by `scale`.
/// - If the retrieved value is an exact integer literal (e.g. `120`), it is treated as absolute pixels.
/// Returns `default` if the field is not present.
pub fn get_relative_field(obj: &gss::Object, path: &[&str], scale: f32, default: f32) -> f32 {
    if let Some(&val) = obj.get::<u32>(path) {
        val as f32
    } else if let Some(&val) = obj.get::<i32>(path) {
        val as f32
    } else if let Some(&val) = obj.get::<f32>(path) {
        val * scale
    } else {
        default
    }
}

/// Retrieves a string property from the stylesheet [`Style`] at the given path.
///
/// Returns `default` if the field is not present.
pub fn get_string_field(gss: &gss::Object, path: &[&str], default: &str) -> String {
    gss.get::<String>(path)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

/// Retrieves a `usize` property from the stylesheet [`Style`] at the given path.
///
/// Automatically converts integer literals and floats into `usize`.
/// Returns `default` if the field is not present or cannot be parsed.
pub fn get_usize_field(obj: &gss::Object, path: &[&str], default: usize) -> usize {
    if let Some(&val) = obj.get::<usize>(path) {
        val
    } else if let Some(&val) = obj.get::<u32>(path) {
        val as usize
    } else if let Some(&val) = obj.get::<i32>(path) {
        val.max(0) as usize
    } else if let Some(&val) = obj.get::<f32>(path) {
        val.max(0.0) as usize
    } else {
        default
    }
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
            1500.0, 100.0, 50.0, 50.0
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
}
