pub use gss;
pub use raylib;

pub const DEFAULT_ROTATION: f32 = 0.0;
pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_SPACING: f32 = 2.0;
pub const DEFAULT_SCALE: f32 = 1.0;
pub const DEBUG_FRAME_LINE_THICK: f32 = 2.0;
pub const MOUSE_CLICK_RADIUS: f32 = 2.0;
pub const DEFAULT_FPS: i32 = 24;

// raylib helpers commit it back later!

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
