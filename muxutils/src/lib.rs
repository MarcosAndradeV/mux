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

pub fn map_color(string: &str) -> raylib::Color {
    match string {
        "light gray" => raylib::LIGHTGRAY,
        "gray" => raylib::GRAY,
        "dark gray" => raylib::DARKGRAY,
        "yellow" => raylib::YELLOW,
        "gold" => raylib::GOLD,
        "orange" => raylib::ORANGE,
        "pink" => raylib::PINK,
        "red" => raylib::RED,
        "maroon" => raylib::MAROON,
        "green" => raylib::GREEN,
        "lime" => raylib::LIME,
        "dark green" => raylib::DARKGREEN,
        "sky blue" => raylib::SKYBLUE,
        "blue" => raylib::BLUE,
        "dark blue" => raylib::DARKBLUE,
        "purple" => raylib::PURPLE,
        "violet" => raylib::VIOLET,
        "dark purple" => raylib::DARKPURPLE,
        "beige" => raylib::BEIGE,
        "brown" => raylib::BROWN,
        "dark brown" => raylib::DARKBROWN,
        "white" => raylib::WHITE,
        "black" => raylib::BLACK,
        "magenta" => raylib::MAGENTA,
        _ => raylib::BLANK,
    }
}

/// Retrieves a color property from the stylesheet [`Style`] at the given path segment slice (e.g., `&["button", "color"]`).
///
/// It supports reading color values defined as string names (e.g. `"red"`, `"dark green"`) or
/// hex codes as a `u32` value (e.g. `0xFF00FFFF`). If the color cannot be found or parsed,
/// the `default` color is returned.
pub fn get_color_field(obj: &gss::Object, path: &[&str], default: raylib::Color) -> raylib::Color {
    if let Some(string) = obj.get::<String>(path) {
        map_color(string)
    } else if let Some(hex) = obj.get::<u32>(path) {
        raylib::get_color(*hex)
    } else {
        default
    }
}

/// Retrieves a boolean property from the stylesheet [`Style`] for a specific element name and field.
///
/// Returns `default` if the field is not present.
pub fn get_bool_field(obj: &gss::Object, name: &str, field: &str, default: bool) -> bool {
    if let Some(&val) = obj.get::<bool>(&[name, field]) {
        val
    } else {
        default
    }
}

/// Retrieves a float (`f32`) property from the stylesheet [`Style`] at the given path.
///
/// It supports reading direct float values or converting unsigned integers (`u32`) to floats.
/// Returns `default` if the field is not present or cannot be retrieved as `f32` or `u32`.
pub fn get_f32_field(obj: &gss::Object, path: &[&str], default: f32) -> f32 {
    if let Some(&val) = obj.get::<f32>(path) {
        val
    } else if let Some(&val) = obj.get::<u32>(path) {
        val as f32
    } else {
        default
    }
}

/// Retrieves a relative coordinate or float property from the stylesheet [`Style`] at the given path.
///
/// If the retrieved value is an `f32`, it is scaled by the provided `scale` factor.
/// If the value is a direct `u32`, it is returned raw without scaling.
/// Returns `default` if the field is not present.
pub fn get_relative_field(obj: &gss::Object, path: &[&str], scale: f32, default: f32) -> f32 {
    if let Some(&val) = obj.get::<f32>(path) {
        val * scale
    } else if let Some(&val) = obj.get::<u32>(path) {
        val as f32
    } else {
        default
    }
}

pub fn get_string_field(gss: &gss::Object, path: &[&str], default: &str) -> String {
    if let Some(val) = gss.get::<String>(path) {
        val.clone()
    } else {
        default.to_string()
    }
}

/// Retrieves a `usize` property from the stylesheet [`Style`] at the given path.
///
/// It supports reading direct integer (`usize`, `u32`, `i32`) or float (`f32`) values converted to `usize`.
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
}
