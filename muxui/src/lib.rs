#![warn(missing_docs)]
//! # muxui
//!
//! A lightweight user interface library built on top of Raylib (`raylib-rs`) and styled
//! using Graph Style Sheets (GSS).
//!
//! ## Overview
//!
//! `muxui` provides a simple, declarative way to lay out and render graphical elements in a window.
//! The library's core structure revolves around:
//! - [`Element`]: A trait representing any UI widget that can be measured, drawn, and handle events.
//! - Standard elements: [`TextElement`], [`TextureElement`], [`RenderTextureElement`], [`RectangleElement`], [`ButtonElement`], [`StackLayout`], and [`GridLayout`].
//! - Styling via [`Style`] (alias for [`Gss`]), which resolves layout configurations like margins, spacing, and colors.

/// Type alias for the Graph Style Sheets ([`Gss`]) context used to style UI components.
pub type Style = muxutils::gss::Gss;

pub use muxutils::raylib::*;
pub use muxutils::*;

/// The interaction response returned when placing or checking interactive elements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Response {
    /// `true` if the element was clicked this frame (mouse left-button pressed while hovering).
    pub clicked: bool,
    /// `true` if the mouse cursor is currently hovering over the element.
    pub hovered: bool,
    /// `true` if the mouse left-button is currently held down on the element.
    pub pressed: bool,
    /// The resolved bounding rectangle of the element.
    pub rect: Rectangle,
}

impl Response {
    /// Returns `true` if the element was clicked this frame.
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` if the element is currently hovered.
    pub fn hovered(&self) -> bool {
        self.hovered
    }

    /// Returns `true` if the element is currently pressed.
    pub fn pressed(&self) -> bool {
        self.pressed
    }

    /// Returns the resolved layout bounding rectangle.
    pub fn rect(&self) -> Rectangle {
        self.rect
    }
}

/// The base trait for all UI elements and layout managers in `muxui`.
///
/// Implementers of this trait can be measured and drawn.
pub trait Element {
    /// Draws the element on the screen at the resolved position.
    ///
    /// # Arguments
    ///
    /// * `position` - The absolute Vector2 coordinate where the element should be rendered.
    /// * `style` - The stylesheet to use for rendering properties (e.g. colors, spacing).
    /// * `name` - The unique stylesheet selector name for this element instance.
    fn draw(&self, position: Vector2, style: &Style, name: &str);

    /// Measures the dimensions of this element under the given style.
    ///
    /// Returns a [`Vector2`] representing the width (`x`) and height (`y`) of the element.
    ///
    /// # Arguments
    ///
    /// * `style` - The stylesheet to use for resolving sizes.
    /// * `name` - The unique stylesheet selector name for this element.
    fn measure(&self, style: &Style, name: &str) -> Vector2;

    /// Calculates the element's position based on its style rules, renders it,
    /// and optionally draws a debug border if configured via style.
    ///
    /// If the boolean field `"frame"` is `true` for this element in the stylesheet,
    /// a green bounding box of thickness `2.0` is drawn around the element.
    ///
    /// # Arguments
    ///
    /// * `style` - The stylesheet context.
    /// * `name` - The unique style selector name.
    fn place(&self, style: &Style, name: &str) {
        let rec = self.get_rec(style, name);
        if is_rect_on_screen(rec) {
            let position = Vector2 { x: rec.x, y: rec.y };
            self.draw(position, style, name);
            if is_window_ready() && get_bool_field(style, name, "frame", false) {
                draw_rectangle_lines_ex(rec, DEBUG_FRAME_LINE_THICK, GREEN);
            }
        }
    }

    /// Computes and returns the layout bounding box ([`Rectangle`]) of the element.
    ///
    /// # Arguments
    ///
    /// * `style` - The stylesheet context.
    /// * `name` - The unique style selector name.
    fn get_rec(&self, style: &Style, name: &str) -> Rectangle {
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

    /// Computes the absolute screen position of this element.
    ///
    /// Resolves properties like `"left"`, `"right"`, `"top"`, `"bottom"`, `"align"`, and `"valign"`
    /// relative to screen dimensions.
    ///
    /// # Arguments
    ///
    /// * `style` - The stylesheet context.
    /// * `name` - The unique style selector name.
    fn get_position(&self, style: &Style, name: &str) -> Vector2 {
        let screen_w = get_virtual_screen_width();
        let screen_h = get_virtual_screen_height();
        let size = self.measure(style, name);

        let mut x = if let Some(right) = get_optional_relative_field(style, &[name, "right"], screen_w) {
            screen_w - right - size.x
        } else {
            get_relative_field(style, &[name, "left"], screen_w, 0.0)
        };

        let mut y = if let Some(bottom) = get_optional_relative_field(style, &[name, "bottom"], screen_h) {
            screen_h - bottom - size.y
        } else {
            get_relative_field(style, &[name, "top"], screen_h, 0.0)
        };

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

/// A standard text element that renders a string of text.
///
/// # GSS Properties
///
/// - `font_size` - The font size of the text (defaults to `20.0`).
/// - `color` - The text color (defaults to `WHITE`).
/// - `spacing` - The character spacing (defaults to `2.0`).
pub struct TextElement {
    /// The string text to display.
    pub text: String,
    font: Font,
}

impl TextElement {
    /// Creates a new `TextElement` with the default font.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font: get_font_default(),
        }
    }

    /// Creates a new `TextElement` with a custom [`Font`].
    pub fn create(text: impl Into<String>, font: Font) -> Self {
        Self {
            text: text.into(),
            font,
        }
    }
}

impl Element for TextElement {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        if !is_window_ready() {
            return;
        }
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

use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::zeroed;

thread_local! {
    static TEXTURE_CACHE: RefCell<HashMap<String, Texture2D>> = RefCell::new(HashMap::new());
}

/// Retrieve a texture from the thread-local cache, loading it from disk if not already cached.
///
/// This is safe and performant for single-threaded rendering in Raylib.
pub fn get_cached_texture(path: &str) -> Texture2D {
    if path.is_empty() {
        return unsafe { zeroed() };
    }
    TEXTURE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(&texture) = cache.get(path) {
            texture
        } else {
            let path_c = cstr!(path);
            let texture = load_texture(path_c);
            if is_texture_valid(texture) {
                log_info!("MUXUI: Automatically cached texture: \"{}\"", path);
            } else {
                log_warn!("MUXUI: Failed to load texture: \"{}\"", path);
            }
            cache.insert(path.to_string(), texture);
            texture
        }
    })
}

/// Clear all cached textures from GPU memory.
pub fn clear_texture_cache() {
    TEXTURE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        log_info!(
            "MUXUI: Clearing texture cache ({} textures)...",
            cache.len()
        );
        for (_, texture) in cache.drain() {
            if is_window_ready() && is_texture_valid(texture) {
                unload_texture(texture);
            }
        }
    });
}

/// A graphical element that wraps a texture loaded from a path.
///
/// Textures are loaded and cached dynamically at runtime using the thread-local
/// texture cache, removing manual memory/lifecycle management from the developer.
///
/// # GSS Properties
///
/// - `scale` - The scaling factor applied to the texture dimensions (defaults to `1.0`).
/// - `rotation` - The rotation angle in degrees (defaults to `0.0`).
/// - `color` - The tint color applied to the texture when drawn (defaults to `WHITE`).
pub struct TextureElement {
    /// Filepath or asset identifier of the texture.
    pub path: String,
}

impl TextureElement {
    /// Creates a new `TextureElement` wrapping the given path.
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// Loads a texture from the given filesystem path (compatibility wrapper).
    pub fn load_from_file(path: impl AsRef<str>) -> Option<Self> {
        let path_str = path.as_ref();
        if path_str.is_empty() {
            None
        } else {
            Some(Self {
                path: path_str.to_string(),
            })
        }
    }

    /// Creates a placeholder, invalid [`TextureElement`] representing an empty/default texture.
    pub fn invalid() -> Self {
        Self {
            path: String::new(),
        }
    }
}

impl Element for TextureElement {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        let texture = get_cached_texture(&self.path);
        if is_texture_valid(texture) {
            let scale = get_f32_field(style, &[name, "scale"], DEFAULT_SCALE);
            let rotation = get_f32_field(style, &[name, "rotation"], DEFAULT_ROTATION);
            let color = get_color_field(style, &[name, "color"], WHITE);
            draw_texture_ex(texture, position, rotation, scale, color);
        }
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        let texture = get_cached_texture(&self.path);
        let scale = get_f32_field(style, &[name, "scale"], DEFAULT_SCALE);
        if is_texture_valid(texture) {
            Vector2 {
                x: texture.width as f32 * scale,
                y: texture.height as f32 * scale,
            }
        } else {
            Vector2::zero()
        }
    }
}

/// A graphical element that displays an offscreen [`RenderTexture`].
///
/// Render textures are useful for offscreen rendering, mini-maps, particle surfaces,
/// or procedural canvases rendered dynamically.
///
/// # GSS Properties
///
/// - `scale` - The scaling factor applied to the render texture dimensions (defaults to `1.0`).
/// - `rotation` - The rotation angle in degrees (defaults to `0.0`).
/// - `color` - The tint color applied to the texture when drawn (defaults to `WHITE`).
/// - `width` - Explicit width override (defaults to scaled texture width).
/// - `height` - Explicit height override (defaults to scaled texture height).
pub struct RenderTextureElement {
    /// The underlying Raylib RenderTexture.
    pub target: RenderTexture,
}

impl RenderTextureElement {
    /// Creates a new `RenderTextureElement` wrapping the given render texture target.
    pub fn new(target: RenderTexture) -> Self {
        Self { target }
    }

    /// Creates an empty placeholder `RenderTextureElement`.
    pub fn invalid() -> Self {
        Self {
            target: unsafe { zeroed() },
        }
    }

    /// Returns the underlying render texture.
    pub fn target(&self) -> RenderTexture {
        self.target
    }

    /// Returns a mutable reference to the underlying render texture.
    pub fn target_mut(&mut self) -> &mut RenderTexture {
        &mut self.target
    }

    /// Sets a new render target, unloading the previous target if valid.
    pub fn set_target(&mut self, target: RenderTexture) {
        self.unload();
        self.target = target;
    }

    /// Unloads the render texture from GPU memory if valid and the window is initialized.
    pub fn unload(&mut self) {
        if is_window_ready() && is_render_texture_valid(self.target) {
            unload_render_texture(self.target);
            self.target = unsafe { zeroed() };
        }
    }
}

impl Element for RenderTextureElement {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        if self.target.id > 0 && is_texture_valid(self.target.texture) {
            let size = self.measure(style, name);
            let rotation = get_f32_field(style, &[name, "rotation"], DEFAULT_ROTATION);
            let color = get_color_field(style, &[name, "color"], WHITE);
            // Raylib render textures are inverted vertically on the Y axis in OpenGL framebuffer
            let source = Rectangle {
                x: 0.0,
                y: 0.0,
                width: self.target.texture.width as f32,
                height: -(self.target.texture.height as f32),
            };
            let dest = Rectangle {
                x: position.x,
                y: position.y,
                width: size.x,
                height: size.y,
            };
            draw_texture_pro(
                self.target.texture,
                source,
                dest,
                Vector2::zero(),
                rotation,
                color,
            );
        }
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        let scale = get_f32_field(style, &[name, "scale"], DEFAULT_SCALE);
        let default_w = if self.target.id > 0 && is_texture_valid(self.target.texture) {
            self.target.texture.width as f32 * scale
        } else {
            0.0
        };
        let default_h = if self.target.id > 0 && is_texture_valid(self.target.texture) {
            self.target.texture.height as f32 * scale
        } else {
            0.0
        };
        let width = get_f32_field(style, &[name, "width"], default_w);
        let height = get_f32_field(style, &[name, "height"], default_h);
        Vector2 {
            x: width,
            y: height,
        }
    }
}

/// A clickable UI button wrapping any [`Element`].
///
/// Buttons can be instantiated on the fly in immediate-mode updates and do **not** need
/// to be stored in the application context. Interaction state (clicked, hovered, pressed)
/// is computed immediately during layout placement or drawing.
///
/// # Standalone Usage
///
/// ```no_run
/// use muxui::*;
///
/// fn update_ui(style: &Style) {
///     let response = Button::new("Click Me").place(style, "my_button");
///     if response.clicked {
///         println!("Button was clicked!");
///     }
/// }
/// ```
///
/// # Inside Layout Containers
///
/// ```no_run
/// use muxui::*;
///
/// fn update_ui(style: &Style) {
///     let play_btn = Button::new("Play");
///     let quit_btn = Button::new("Quit");
///
///     StackLayout::new(&[
///         ("play", &play_btn),
///         ("quit", &quit_btn),
///     ]).place(style, "menu_stack");
///
///     if play_btn.clicked() {
///         println!("Play clicked!");
///     }
///     if quit_btn.clicked() {
///         println!("Quit clicked!");
///     }
/// }
/// ```
///
/// # GSS Properties
///
/// - `bg` / `background_color` - Base background color (defaults to `BLANK`).
/// - `hover_color` / `button.hover_color` - Highlight color when hovered (defaults to `0xFFFFFF33`).
/// - `active_color` / `pressed_color` - Highlight color when pressed (defaults to `0xFFFFFF55`).
/// - `disabled_color` / `disabled_bg` - Dim overlay color when disabled (defaults to `0x00000088`).
/// - `width` - Explicit button width (pixels or percentage, e.g. `120` or `30%`).
/// - `height` - Explicit button height (pixels or percentage, e.g. `40` or `10%`).
/// - `padding` / `padding_x`, `padding_y` - Internal padding around element content.
pub struct Button<E: Element> {
    element: E,
    disabled: bool,
    clicked: std::cell::Cell<bool>,
    hovered: std::cell::Cell<bool>,
    pressed: std::cell::Cell<bool>,
    rect: std::cell::Cell<Rectangle>,
}

impl Button<TextElement> {
    /// Creates a new `Button` displaying a text label.
    pub fn new(label: impl Into<String>) -> Self {
        Self::from_element(TextElement::new(label))
    }
}

impl<E: Element> Button<E> {
    /// Creates a new `Button` wrapping any underlying UI [`Element`].
    pub fn from_element(element: E) -> Self {
        Self {
            element,
            disabled: false,
            clicked: std::cell::Cell::new(false),
            hovered: std::cell::Cell::new(false),
            pressed: std::cell::Cell::new(false),
            rect: std::cell::Cell::new(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            }),
        }
    }

    /// Backwards-compatible constructor wrapping an element.
    pub fn new_with_no_cached_rec(element: E) -> Self {
        Self::from_element(element)
    }

    /// Builder method to set the disabled state of the button.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the disabled state of the button.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns `true` if the button is currently disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns `true` if the button was clicked this frame.
    pub fn clicked(&self) -> bool {
        self.clicked.get()
    }

    /// Compatibility alias for [`clicked`].
    pub fn click(&self) -> bool {
        self.clicked()
    }

    /// Returns `true` if the button is currently hovered.
    pub fn hovered(&self) -> bool {
        self.hovered.get()
    }

    /// Compatibility alias for [`hovered`].
    pub fn hover(&self) -> bool {
        self.hovered()
    }

    /// Returns `true` if the button is currently pressed.
    pub fn pressed(&self) -> bool {
        self.pressed.get()
    }

    /// Returns the resolved bounding rectangle of the button.
    pub fn rect(&self) -> Rectangle {
        self.rect.get()
    }

    /// Checks if the button was clicked via mouse or a specific keyboard shortcut.
    pub fn clicked_or_key(&self, key: KeyboardKey) -> bool {
        if self.disabled {
            return false;
        }
        self.clicked() || is_key_pressed(key)
    }

    /// Compatibility alias for [`clicked_or_key`].
    pub fn click_or_key(&self, key: KeyboardKey) -> bool {
        self.clicked_or_key(key)
    }

    /// Returns an immutable reference to the wrapped element.
    pub fn element(&self) -> &E {
        &self.element
    }

    /// Returns a mutable reference to the wrapped element.
    pub fn element_mut(&mut self) -> &mut E {
        &mut self.element
    }

    /// Consumes the button and returns the inner element.
    pub fn into_inner(self) -> E {
        self.element
    }

    /// Measures, positions, and draws the button, returning an interaction [`Response`].
    pub fn place(&self, style: &Style, name: &str) -> Response {
        let rec = self.get_rec(style, name);
        if is_rect_on_screen(rec) {
            let position = Vector2 { x: rec.x, y: rec.y };
            self.draw(position, style, name);
            if is_window_ready() && get_bool_field(style, name, "frame", false) {
                draw_rectangle_lines_ex(rec, DEBUG_FRAME_LINE_THICK, GREEN);
            }
        }
        Response {
            clicked: self.clicked.get(),
            hovered: self.hovered.get(),
            pressed: self.pressed.get(),
            rect: self.rect.get(),
        }
    }
}

impl<E: Element> Element for Button<E> {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        let size = self.measure(style, name);
        let rec = Rectangle {
            x: position.x,
            y: position.y,
            width: size.x,
            height: size.y,
        };
        self.rect.set(rec);

        // Hit-test against current frame virtual mouse coordinates
        let mouse_pos = get_virtual_mouse_position();
        let is_hovered = !self.disabled && check_collision_point_rec(mouse_pos, rec);
        let is_pressed = is_hovered && is_mouse_button_down(MOUSE_BUTTON_LEFT);
        let is_clicked = is_hovered && is_mouse_button_pressed(MOUSE_BUTTON_LEFT);

        self.hovered.set(is_hovered);
        self.pressed.set(is_pressed);
        self.clicked.set(is_clicked);

        let window_ready = is_window_ready();

        // Background color
        let bg_color = get_color_field(
            style,
            &[name, "background_color"],
            get_color_field(style, &[name, "bg"], BLANK),
        );
        if window_ready && bg_color != BLANK {
            draw_rectangle_rec(rec, bg_color);
        }

        // Active / Pressed overlay
        if window_ready && is_pressed {
            let active_color = get_color_field(
                style,
                &[name, "active_color"],
                get_color_field(
                    style,
                    &[name, "pressed_color"],
                    get_color(0xFFFFFF55),
                ),
            );
            if active_color != BLANK {
                draw_rectangle_rec(rec, active_color);
            }
        } else if window_ready && is_hovered {
            // Hover overlay
            let hover_color = get_color_field(
                style,
                &[name, "hover_color"],
                get_color_field(
                    style,
                    &[name, "button", "hover_color"],
                    get_color(0xFFFFFF33),
                ),
            );
            if hover_color != BLANK {
                draw_rectangle_rec(rec, hover_color);
            }
        }

        // Center inner element if button has explicit width/height
        let inner_size = self.element.measure(style, name);
        let inner_x = position.x + ((size.x - inner_size.x) / 2.0).max(0.0);
        let inner_y = position.y + ((size.y - inner_size.y) / 2.0).max(0.0);

        self.element.draw(Vector2 { x: inner_x, y: inner_y }, style, name);

        // Disabled overlay
        if window_ready && self.disabled {
            let disabled_color = get_color_field(
                style,
                &[name, "disabled_color"],
                get_color_field(style, &[name, "disabled_bg"], get_color(0x00000088)),
            );
            if disabled_color != BLANK {
                draw_rectangle_rec(rec, disabled_color);
            }
        }
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        let inner_size = self.element.measure(style, name);
        let screen_w = get_virtual_screen_width();
        let screen_h = get_virtual_screen_height();

        let pad = get_f32_field(style, &[name, "padding"], 0.0);
        let pad_x = get_f32_field(style, &[name, "padding_x"], pad);
        let pad_y = get_f32_field(style, &[name, "padding_y"], pad);

        let default_w = inner_size.x + pad_x * 2.0;
        let default_h = inner_size.y + pad_y * 2.0;

        let width = get_optional_relative_field(style, &[name, "width"], screen_w)
            .unwrap_or(default_w);
        let height = get_optional_relative_field(style, &[name, "height"], screen_h)
            .unwrap_or(default_h);

        Vector2 {
            x: width.max(default_w),
            y: height.max(default_h),
        }
    }
}

/// Backwards-compatible type alias for [`Button`].
pub type ButtonElement<E> = Button<E>;

/// Places a text button on the screen styled under `name`, returning an interaction [`Response`].
pub fn button(style: &Style, name: &str, label: impl Into<String>) -> Response {
    Button::new(label).place(style, name)
}


/// A layout container that arranges its child elements sequentially in a single direction (vertical or horizontal).
///
/// # GSS Properties
///
/// - `direction` - The layout direction, either `"vertical"` or `"horizontal"` (defaults to `"vertical"`).
/// - `gap` - The gap spacing between sequential elements (defaults to `0.0`).
///
/// # Child Properties
///
/// Child elements placed inside the stack can be styled on the cross-axis with these properties:
/// - `align` - Horizontal alignment for child elements in a vertical layout (`"center"`, `"right"`).
/// - `valign` - Vertical alignment for child elements in a horizontal layout (`"middle"`, `"center"`, `"bottom"`).
/// - `frame` - Draws a green debug border around the child if `true`.
pub struct StackLayout<'a, 'b> {
    children: &'b [(&'a str, &'a dyn Element)],
}

impl<'a, 'b> StackLayout<'a, 'b> {
    /// Creates a new `StackLayout` with the specified named child elements.
    ///
    /// Each child is specified as a tuple of `(selector_name, element_ref)`.
    pub fn new(children: &'b [(&'a str, &'a dyn Element)]) -> Self {
        Self { children }
    }

    /// Returns the slice of children elements managed by this layout.
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

            let rec = Rectangle {
                x: child_pos.x,
                y: child_pos.y,
                width: child_size.x,
                height: child_size.y,
            };

            if is_rect_on_screen(rec) {
                child.draw(child_pos, style, child_name);

                if is_window_ready() && get_bool_field(style, child_name, "frame", false) {
                    draw_rectangle_lines_ex(rec, DEBUG_FRAME_LINE_THICK, GREEN);
                }
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

/// A 2D grid layout container that arranges child elements across rows and columns.
///
/// # GSS Properties
///
/// - `columns` - The number of columns in the grid (defaults to `2`).
/// - `gap` - Default spacing applied between rows and columns (defaults to `0.0`).
/// - `column_gap` - Spacing between adjacent columns (defaults to `gap` or `0.0`).
/// - `row_gap` - Spacing between adjacent rows (defaults to `gap` or `0.0`).
/// - `cell_width` - Explicit fixed cell width override (defaults to maximum measured child width).
/// - `cell_height` - Explicit fixed cell height override (defaults to maximum measured child height).
///
/// # Child Properties
///
/// Child elements placed inside the grid can specify alignment inside their allocated cell:
/// - `align` - Horizontal alignment within cell (`"center"`, `"right"`).
/// - `valign` - Vertical alignment within cell (`"middle"`, `"center"`, `"bottom"`).
/// - `frame` - Draws a green debug border around the child if `true`.
pub struct GridLayout<'a, 'b> {
    children: &'b [(&'a str, &'a dyn Element)],
}

impl<'a, 'b> GridLayout<'a, 'b> {
    /// Creates a new `GridLayout` with the specified named child elements.
    ///
    /// Each child is specified as a tuple of `(selector_name, element_ref)`.
    pub fn new(children: &'b [(&'a str, &'a dyn Element)]) -> Self {
        Self { children }
    }

    /// Returns the slice of children elements managed by this layout.
    pub fn children(&self) -> &[(&'a str, &'a dyn Element)] {
        &self.children
    }

    /// Computes the effective cell dimensions `(cell_width, cell_height)` for the grid.
    pub fn get_cell_size(&self, style: &Style, name: &str) -> Vector2 {
        let explicit_w = style
            .get::<f32>(&[name, "cell_width"])
            .copied()
            .or_else(|| style.get::<u32>(&[name, "cell_width"]).map(|&v| v as f32));
        let explicit_h = style
            .get::<f32>(&[name, "cell_height"])
            .copied()
            .or_else(|| style.get::<u32>(&[name, "cell_height"]).map(|&v| v as f32));

        let mut max_w: f32 = 0.0;
        let mut max_h: f32 = 0.0;

        if explicit_w.is_none() || explicit_h.is_none() {
            for (child_name, child) in self.children {
                let size = child.measure(style, child_name);
                max_w = max_w.max(size.x);
                max_h = max_h.max(size.y);
            }
        }

        Vector2 {
            x: explicit_w.unwrap_or(max_w),
            y: explicit_h.unwrap_or(max_h),
        }
    }
}

impl<'a, 'b> Element for GridLayout<'a, 'b> {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        if self.children.is_empty() {
            return;
        }

        let columns = get_usize_field(style, &[name, "columns"], 2).max(1);
        let default_gap = get_f32_field(style, &[name, "gap"], 0.0);
        let col_gap = get_f32_field(style, &[name, "column_gap"], default_gap);
        let row_gap = get_f32_field(style, &[name, "row_gap"], default_gap);
        let cell_size = self.get_cell_size(style, name);

        for (i, (child_name, child)) in self.children.iter().enumerate() {
            let col = (i % columns) as f32;
            let row = (i / columns) as f32;

            let child_size = child.measure(style, child_name);
            let mut child_x = position.x + col * (cell_size.x + col_gap);
            let mut child_y = position.y + row * (cell_size.y + row_gap);

            // Optional child alignment within cell
            if let Some(align) = style.get::<String>(&[child_name, "align"]) {
                match align.as_str() {
                    "center" => child_x += (cell_size.x - child_size.x) / 2.0,
                    "right" => child_x += cell_size.x - child_size.x,
                    _ => {}
                }
            }

            if let Some(valign) = style.get::<String>(&[child_name, "valign"]) {
                match valign.as_str() {
                    "middle" | "center" => child_y += (cell_size.y - child_size.y) / 2.0,
                    "bottom" => child_y += cell_size.y - child_size.y,
                    _ => {}
                }
            }

            let child_pos = Vector2 {
                x: child_x,
                y: child_y,
            };

            let rec = Rectangle {
                x: child_pos.x,
                y: child_pos.y,
                width: child_size.x,
                height: child_size.y,
            };

            if is_rect_on_screen(rec) {
                child.draw(child_pos, style, child_name);

                if is_window_ready() && get_bool_field(style, child_name, "frame", false) {
                    draw_rectangle_lines_ex(rec, DEBUG_FRAME_LINE_THICK, GREEN);
                }
            }
        }
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        if self.children.is_empty() {
            return Vector2::zero();
        }

        let columns = get_usize_field(style, &[name, "columns"], 2).max(1);
        let default_gap = get_f32_field(style, &[name, "gap"], 0.0);
        let col_gap = get_f32_field(style, &[name, "column_gap"], default_gap);
        let row_gap = get_f32_field(style, &[name, "row_gap"], default_gap);
        let cell_size = self.get_cell_size(style, name);

        let total_items = self.children.len();
        let cols_used = total_items.min(columns) as f32;
        let rows_used = ((total_items + columns - 1) / columns) as f32;

        let width = if cols_used > 0.0 {
            cols_used * cell_size.x + (cols_used - 1.0) * col_gap
        } else {
            0.0
        };

        let height = if rows_used > 0.0 {
            rows_used * cell_size.y + (rows_used - 1.0) * row_gap
        } else {
            0.0
        };

        Vector2 {
            x: width,
            y: height,
        }
    }
}

/// A basic layout rectangle that fills space with specified dimensions and background color.
///
/// # GSS Properties
///
/// - `width` - The width of the rectangle (defaults to `0.0`).
/// - `height` - The height of the rectangle (defaults to `0.0`).
/// - `color` - The fill color of the rectangle (defaults to `MAGENTA`).
pub struct RectangleElement;

impl Element for RectangleElement {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        if !is_window_ready() {
            return;
        }
        draw_rectangle_v(
            position,
            self.measure(style, name),
            get_color_field(style, &[name, "color"], MAGENTA),
        );
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        let screen_w = get_virtual_screen_width();
        let screen_h = get_virtual_screen_height();
        Vector2::new(
            get_relative_field(style, &[name, "width"], screen_w, 0.0),
            get_relative_field(style, &[name, "height"], screen_h, 0.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gss::parse_str;

    #[test]
    fn test_style_get_color_field() {
        let style = parse_str(
            r#"
            btn_red = {
                color = "red",
            },
            btn_hex = {
                color = 0xFF00FFFF,
            },
            btn_missing = {},
            "#,
        )
        .unwrap();

        assert_eq!(get_color_field(&style, &["btn_red", "color"], WHITE), RED);
        assert_eq!(
            get_color_field(&style, &["btn_hex", "color"], WHITE),
            get_color(0xFF00FFFF)
        );
        assert_eq!(
            get_color_field(&style, &["btn_missing", "color"], WHITE),
            WHITE
        );
    }

    #[test]
    fn test_style_get_bool_field() {
        let style = parse_str(
            r#"
            widget = {
                visible = true,
            },
            "#,
        )
        .unwrap();

        assert!(get_bool_field(&style, "widget", "visible", false));
        assert!(!get_bool_field(&style, "widget", "missing", false));
        assert!(get_bool_field(&style, "widget", "missing", true));
    }

    #[test]
    fn test_style_get_f32_field() {
        let style = parse_str(
            r#"
            widget = {
                size_float = 24.5,
                size_int = 100,
            },
            "#,
        )
        .unwrap();

        assert_eq!(get_f32_field(&style, &["widget", "size_float"], 0.0), 24.5);
        assert_eq!(get_f32_field(&style, &["widget", "size_int"], 0.0), 100.0);
        assert_eq!(get_f32_field(&style, &["widget", "missing"], 5.0), 5.0);
    }

    #[test]
    fn test_style_get_relative_field() {
        let style = parse_str(
            r#"
            widget = {
                pct = 0.5,
                pixels = 120,
            },
            "#,
        )
        .unwrap();

        assert_eq!(
            get_relative_field(&style, &["widget", "pct"], 800.0, 0.0),
            400.0
        );
        assert_eq!(
            get_relative_field(&style, &["widget", "pixels"], 800.0, 0.0),
            120.0
        );
        assert_eq!(
            get_relative_field(&style, &["widget", "missing"], 800.0, 10.0),
            10.0
        );
    }

    #[test]
    fn test_rectangle_element_measure() {
        set_viewport(800.0, 600.0, 800.0, 600.0);
        let style = parse_str(
            r#"
            rect = {
                width = 150,
                height = 75,
            },
            rect_pct = {
                width = 50%,
                height = 25%,
            },
            "#,
        )
        .unwrap();

        let element = RectangleElement;
        let size = element.measure(&style, "rect");
        assert_eq!(size.x, 150.0);
        assert_eq!(size.y, 75.0);

        let size_pct = element.measure(&style, "rect_pct");
        assert_eq!(size_pct.x, 400.0);
        assert_eq!(size_pct.y, 150.0);
    }

    use std::cell::RefCell;

    struct MockElement {
        size: Vector2,
        draw_positions: RefCell<Vec<Vector2>>,
    }

    impl MockElement {
        fn new(width: f32, height: f32) -> Self {
            Self {
                size: Vector2::new(width, height),
                draw_positions: RefCell::new(Vec::new()),
            }
        }
    }

    impl Element for MockElement {
        fn draw(&self, position: Vector2, _style: &Style, _name: &str) {
            self.draw_positions.borrow_mut().push(position);
        }
        fn measure(&self, _style: &Style, _name: &str) -> Vector2 {
            self.size
        }
    }

    #[test]
    fn test_button_element_wrapping() {
        let child = MockElement::new(60.0, 30.0);
        let mut btn = ButtonElement::new_with_no_cached_rec(child);

        let style = Style::new();
        // Verify size measurement delegates to wrapped element
        assert_eq!(btn.measure(&style, "btn").x, 60.0);
        assert_eq!(btn.measure(&style, "btn").y, 30.0);

        assert_eq!(btn.element().size.x, 60.0);
        assert_eq!(btn.element().size.y, 30.0);

        btn.element_mut().size = Vector2::new(80.0, 40.0);
        assert_eq!(btn.element().size.x, 80.0);

        // Verify updated size measurement delegates correctly
        assert_eq!(btn.measure(&style, "btn").x, 80.0);
        assert_eq!(btn.measure(&style, "btn").y, 40.0);
    }

    #[test]
    fn test_stack_layout_measure_vertical() {
        let style = parse_str(
            r#"
            layout = {
                direction = "vertical",
                gap = 10.0,
            },
            "#,
        )
        .unwrap();

        let child1 = MockElement::new(50.0, 30.0);
        let child2 = MockElement::new(70.0, 40.0);
        let children: &[(&str, &dyn Element)] = &[("child1", &child1), ("child2", &child2)];

        let stack = StackLayout::new(children);
        let size = stack.measure(&style, "layout");

        // Vertical stack: width is max(child_width) = 70.0
        // height is sum(child_height) + gap = 30 + 40 + 10 = 80.0
        assert_eq!(size.x, 70.0);
        assert_eq!(size.y, 80.0);
    }

    #[test]
    fn test_stack_layout_measure_horizontal() {
        let style = parse_str(
            r#"
            layout = {
                direction = "horizontal",
                gap = 5.0,
            },
            "#,
        )
        .unwrap();

        let child1 = MockElement::new(50.0, 30.0);
        let child2 = MockElement::new(70.0, 40.0);
        let children: &[(&str, &dyn Element)] = &[("child1", &child1), ("child2", &child2)];

        let stack = StackLayout::new(children);
        let size = stack.measure(&style, "layout");

        // Horizontal stack: width is sum(child_width) + gap = 50 + 70 + 5 = 125.0
        // height is max(child_height) = 40.0
        assert_eq!(size.x, 125.0);
        assert_eq!(size.y, 40.0);
    }

    #[test]
    fn test_stack_layout_positioning_vertical() {
        let style = parse_str(
            r#"
            layout = {
                direction = "vertical",
                gap = 10.0,
            },
            child1 = {},
            child2 = {
                align = "center",
            },
            child3 = {
                align = "right",
            },
            "#,
        )
        .unwrap();

        let child1 = MockElement::new(50.0, 30.0);
        let child2 = MockElement::new(40.0, 20.0);
        let child3 = MockElement::new(30.0, 10.0);
        let children: &[(&str, &dyn Element)] = &[
            ("child1", &child1),
            ("child2", &child2),
            ("child3", &child3),
        ];

        let stack = StackLayout::new(children);
        let base_pos = Vector2::new(100.0, 100.0);
        stack.draw(base_pos, &style, "layout");

        // Verify draw positions
        let pos1 = child1.draw_positions.borrow()[0];
        let pos2 = child2.draw_positions.borrow()[0];
        let pos3 = child3.draw_positions.borrow()[0];

        // child 1: base position (default top=0, left=0 relative to base, align default is left)
        assert_eq!(pos1.x, 100.0);
        assert_eq!(pos1.y, 100.0);

        // child 2: base y + height1 + gap = 100 + 30 + 10 = 140.0. x centered: 100 + (50 - 40) / 2 = 105.0
        assert_eq!(pos2.x, 105.0);
        assert_eq!(pos2.y, 140.0);

        // child 3: base y + height1 + height2 + 2*gap = 140 + 20 + 10 = 170.0. x right: 100 + 50 - 30 = 120.0
        assert_eq!(pos3.x, 120.0);
        assert_eq!(pos3.y, 170.0);
    }

    #[test]
    fn test_stack_layout_positioning_horizontal() {
        let style = parse_str(
            r#"
            layout = {
                direction = "horizontal",
                gap = 5.0,
            },
            child1 = {},
            child2 = {
                valign = "center",
            },
            child3 = {
                valign = "bottom",
            },
            "#,
        )
        .unwrap();

        let child1 = MockElement::new(30.0, 50.0);
        let child2 = MockElement::new(20.0, 40.0);
        let child3 = MockElement::new(10.0, 30.0);
        let children: &[(&str, &dyn Element)] = &[
            ("child1", &child1),
            ("child2", &child2),
            ("child3", &child3),
        ];

        let stack = StackLayout::new(children);
        let base_pos = Vector2::new(100.0, 100.0);
        stack.draw(base_pos, &style, "layout");

        // Verify draw positions
        let pos1 = child1.draw_positions.borrow()[0];
        let pos2 = child2.draw_positions.borrow()[0];
        let pos3 = child3.draw_positions.borrow()[0];

        // child 1: base position
        assert_eq!(pos1.x, 100.0);
        assert_eq!(pos1.y, 100.0);

        // child 2: base x + width1 + gap = 100 + 30 + 5 = 135.0. y centered: 100 + (50 - 40) / 2 = 105.0
        assert_eq!(pos2.x, 135.0);
        assert_eq!(pos2.y, 105.0);

        // child 3: base x + width1 + width2 + 2*gap = 135 + 20 + 5 = 160.0. y bottom: 100 + 50 - 30 = 120.0
        assert_eq!(pos3.x, 160.0);
        assert_eq!(pos3.y, 120.0);
    }

    #[test]
    fn test_stack_layout_defaults_and_edge_cases() {
        // Missing GSS layout settings (should default to vertical and gap 0.0)
        let style = parse_str(
            r#"
            layout = {},
            "#,
        )
        .unwrap();

        let child1 = MockElement::new(10.0, 20.0);
        let child2 = MockElement::new(15.0, 30.0);

        // 1. Test defaults measurement
        {
            let children: &[(&str, &dyn Element)] = &[("child1", &child1), ("child2", &child2)];
            let stack = StackLayout::new(children);
            let size = stack.measure(&style, "layout");
            // Vertical default: max width = 15.0, sum height = 50.0
            assert_eq!(size.x, 15.0);
            assert_eq!(size.y, 50.0);
        }

        // 2. Test 0 children edge case
        {
            let children: &[(&str, &dyn Element)] = &[];
            let stack = StackLayout::new(children);
            let size = stack.measure(&style, "layout");
            assert_eq!(size.x, 0.0);
            assert_eq!(size.y, 0.0);
        }

        // 3. Test 1 child edge case
        {
            let children: &[(&str, &dyn Element)] = &[("child1", &child1)];
            let stack = StackLayout::new(children);
            let size = stack.measure(&style, "layout");
            assert_eq!(size.x, 10.0);
            assert_eq!(size.y, 20.0);
        }
    }

    #[test]
    fn test_style_get_usize_field() {
        let style = parse_str(
            r#"
            grid_config = {
                cols_int = 4,
                cols_float = 3.0,
            },
            "#,
        )
        .unwrap();

        assert_eq!(get_usize_field(&style, &["grid_config", "cols_int"], 2), 4);
        assert_eq!(
            get_usize_field(&style, &["grid_config", "cols_float"], 2),
            3
        );
        assert_eq!(get_usize_field(&style, &["grid_config", "missing"], 2), 2);
    }

    #[test]
    fn test_button_element_disabled_state() {
        let child = MockElement::new(60.0, 30.0);
        let mut btn = ButtonElement::new_with_no_cached_rec(child);

        assert!(!btn.is_disabled());

        btn.set_disabled(true);
        assert!(btn.is_disabled());

        btn.set_disabled(false);
        assert!(!btn.is_disabled());

        let btn_disabled = btn.with_disabled(true);
        assert!(btn_disabled.is_disabled());

        let inner = btn_disabled.into_inner();
        assert_eq!(inner.size.x, 60.0);
    }

    #[test]
    fn test_grid_layout_measure_basic() {
        let style = parse_str(
            r#"
            grid = {
                columns = 2,
                gap = 10.0,
            },
            "#,
        )
        .unwrap();

        let child1 = MockElement::new(50.0, 30.0);
        let child2 = MockElement::new(60.0, 40.0);
        let child3 = MockElement::new(40.0, 20.0);
        let child4 = MockElement::new(55.0, 35.0);
        let children: &[(&str, &dyn Element)] = &[
            ("child1", &child1),
            ("child2", &child2),
            ("child3", &child3),
            ("child4", &child4),
        ];

        let grid = GridLayout::new(children);
        let size = grid.measure(&style, "grid");

        // Cell width = max(50, 60, 40, 55) = 60.0
        // Cell height = max(30, 40, 20, 35) = 40.0
        // 4 items in 2 columns => 2 cols, 2 rows
        // Width: 2 * 60 + 1 * 10 = 130.0
        // Height: 2 * 40 + 1 * 10 = 90.0
        assert_eq!(size.x, 130.0);
        assert_eq!(size.y, 90.0);
    }

    #[test]
    fn test_grid_layout_measure_with_cell_overrides() {
        let style = parse_str(
            r#"
            grid = {
                columns = 3,
                cell_width = 100.0,
                cell_height = 50.0,
                column_gap = 12.0,
                row_gap = 8.0,
            },
            "#,
        )
        .unwrap();

        let c1 = MockElement::new(20.0, 20.0);
        let c2 = MockElement::new(20.0, 20.0);
        let c3 = MockElement::new(20.0, 20.0);
        let c4 = MockElement::new(20.0, 20.0);
        let c5 = MockElement::new(20.0, 20.0);
        let children: &[(&str, &dyn Element)] = &[
            ("c1", &c1),
            ("c2", &c2),
            ("c3", &c3),
            ("c4", &c4),
            ("c5", &c5),
        ];

        let grid = GridLayout::new(children);
        let size = grid.measure(&style, "grid");

        // 5 items in 3 columns => 3 cols, 2 rows
        // Width: 3 * 100 + 2 * 12 = 324.0
        // Height: 2 * 50 + 1 * 8 = 108.0
        assert_eq!(size.x, 324.0);
        assert_eq!(size.y, 108.0);
    }

    #[test]
    fn test_grid_layout_positioning_and_alignment() {
        let style = parse_str(
            r#"
            grid = {
                columns = 2,
                column_gap = 20.0,
                row_gap = 10.0,
                cell_width = 100.0,
                cell_height = 60.0,
            },
            child1 = {},
            child2 = {
                align = "center",
            },
            child3 = {
                valign = "center",
            },
            child4 = {
                align = "right",
                valign = "bottom",
            },
            "#,
        )
        .unwrap();

        let child1 = MockElement::new(40.0, 20.0);
        let child2 = MockElement::new(40.0, 20.0);
        let child3 = MockElement::new(40.0, 20.0);
        let child4 = MockElement::new(40.0, 20.0);
        let children: &[(&str, &dyn Element)] = &[
            ("child1", &child1),
            ("child2", &child2),
            ("child3", &child3),
            ("child4", &child4),
        ];

        let grid = GridLayout::new(children);
        let base_pos = Vector2::new(50.0, 50.0);
        grid.draw(base_pos, &style, "grid");

        let pos1 = child1.draw_positions.borrow()[0];
        let pos2 = child2.draw_positions.borrow()[0];
        let pos3 = child3.draw_positions.borrow()[0];
        let pos4 = child4.draw_positions.borrow()[0];

        // child1 (row 0, col 0): base (50, 50)
        assert_eq!(pos1.x, 50.0);
        assert_eq!(pos1.y, 50.0);

        // child2 (row 0, col 1): col 1 x = 50 + 100 + 20 = 170.0; align center: 170 + (100 - 40)/2 = 200.0, y = 50.0
        assert_eq!(pos2.x, 200.0);
        assert_eq!(pos2.y, 50.0);

        // child3 (row 1, col 0): col 0 x = 50.0; row 1 y = 50 + 60 + 10 = 120.0; valign center: 120 + (60 - 20)/2 = 140.0
        assert_eq!(pos3.x, 50.0);
        assert_eq!(pos3.y, 140.0);

        // child4 (row 1, col 1): col 1 x = 170.0; align right: 170 + 100 - 40 = 230.0; row 1 y = 120.0; valign bottom: 120 + 60 - 20 = 160.0
        assert_eq!(pos4.x, 230.0);
        assert_eq!(pos4.y, 160.0);
    }

    #[test]
    fn test_grid_layout_edge_cases() {
        let style = parse_str(
            r#"
            grid = {
                columns = 2,
                gap = 5.0,
            },
            "#,
        )
        .unwrap();

        // 0 children
        {
            let children: &[(&str, &dyn Element)] = &[];
            let grid = GridLayout::new(children);
            let size = grid.measure(&style, "grid");
            assert_eq!(size.x, 0.0);
            assert_eq!(size.y, 0.0);
        }

        // 1 child
        {
            let c1 = MockElement::new(30.0, 25.0);
            let children: &[(&str, &dyn Element)] = &[("c1", &c1)];
            let grid = GridLayout::new(children);
            let size = grid.measure(&style, "grid");
            assert_eq!(size.x, 30.0);
            assert_eq!(size.y, 25.0);
        }

        // 3 children in 2 cols (odd items)
        {
            let c1 = MockElement::new(30.0, 20.0);
            let c2 = MockElement::new(30.0, 20.0);
            let c3 = MockElement::new(30.0, 20.0);
            let children: &[(&str, &dyn Element)] = &[("c1", &c1), ("c2", &c2), ("c3", &c3)];
            let grid = GridLayout::new(children);
            let size = grid.measure(&style, "grid");
            // 2 cols: 2 * 30 + 5 = 65.0
            // 2 rows: 2 * 20 + 5 = 45.0
            assert_eq!(size.x, 65.0);
            assert_eq!(size.y, 45.0);
        }
    }

    #[test]
    fn test_element_culling_on_and_off_screen() {
        set_viewport(800.0, 600.0, 800.0, 600.0);
        let style = parse_str(
            r#"
            visible_elem = {
                left = 100,
                top = 100,
            },
            offscreen_elem = {
                left = 2000,
                top = 2000,
            },
            "#,
        )
        .unwrap();

        let elem_on = MockElement::new(50.0, 50.0);
        elem_on.place(&style, "visible_elem");
        assert_eq!(elem_on.draw_positions.borrow().len(), 1);

        let elem_off = MockElement::new(50.0, 50.0);
        elem_off.place(&style, "offscreen_elem");
        assert_eq!(elem_off.draw_positions.borrow().len(), 0);
    }

    #[test]
    fn test_render_texture_element_measure_and_style() {
        let style = parse_str(
            r#"
            rt_default = {},
            rt_scaled = {
                scale = 2.0,
            },
            rt_explicit = {
                width = 512.0,
                height = 256.0,
            },
            "#,
        )
        .unwrap();

        let target = RenderTexture {
            id: 1,
            texture: Texture {
                id: 1,
                width: 100,
                height: 100,
                mipmaps: 1,
                format: 7,
            },
            depth: unsafe { std::mem::zeroed() },
        };

        let elem = RenderTextureElement::new(target);

        // Default measure should match texture dimensions
        let size_default = elem.measure(&style, "rt_default");
        assert_eq!(size_default.x, 100.0);
        assert_eq!(size_default.y, 100.0);

        // Scaled measure
        let size_scaled = elem.measure(&style, "rt_scaled");
        assert_eq!(size_scaled.x, 200.0);
        assert_eq!(size_scaled.y, 200.0);

        // Explicit width/height override
        let size_explicit = elem.measure(&style, "rt_explicit");
        assert_eq!(size_explicit.x, 512.0);
        assert_eq!(size_explicit.y, 256.0);

        // Invalid element defaults to 0x0 unless explicit width/height
        let invalid_elem = RenderTextureElement::invalid();
        assert_eq!(invalid_elem.measure(&style, "rt_default").x, 0.0);
        assert_eq!(invalid_elem.measure(&style, "rt_explicit").x, 512.0);
    }

    #[test]
    fn test_element_positioning_and_alignments() {
        set_viewport(800.0, 600.0, 800.0, 600.0);
        let style = parse_str(
            r#"
            left_top = {
                left = 100,
                top = 50,
            },
            center_valign = {
                left = 200,
                top = 200,
                align = "center",
                valign = "center",
            },
            right_bottom = {
                left = 300,
                top = 400,
                align = "right",
                valign = "bottom",
            },
            valign_middle = {
                left = 100,
                top = 100,
                valign = "middle",
            },
            edge_right_bottom = {
                right = 50,
                bottom = 40,
            },
            edge_pct = {
                right = 10%,
                bottom = 20%,
            },
            "#,
        )
        .unwrap();

        let elem = MockElement::new(40.0, 20.0);
        let pos1 = elem.get_position(&style, "left_top");
        assert_eq!(pos1.x, 100.0);
        assert_eq!(pos1.y, 50.0);
        let rec1 = elem.get_rec(&style, "left_top");
        assert_eq!(rec1.x, 100.0);
        assert_eq!(rec1.y, 50.0);
        assert_eq!(rec1.width, 40.0);
        assert_eq!(rec1.height, 20.0);

        // center: x -= 40/2 = 20 -> 180.0; valign center: y -= 20/2 = 10 -> 190.0
        let pos2 = elem.get_position(&style, "center_valign");
        assert_eq!(pos2.x, 180.0);
        assert_eq!(pos2.y, 190.0);

        // right: x -= 40 -> 260.0; valign bottom: y -= 20 -> 380.0
        let pos3 = elem.get_position(&style, "right_bottom");
        assert_eq!(pos3.x, 260.0);
        assert_eq!(pos3.y, 380.0);

        // valign middle: y -= 20/2 = 10 -> 90.0
        let pos4 = elem.get_position(&style, "valign_middle");
        assert_eq!(pos4.x, 100.0);
        assert_eq!(pos4.y, 90.0);

        // edge_right_bottom: 800 - 50 - 40 = 710.0; 600 - 40 - 20 = 540.0
        let pos5 = elem.get_position(&style, "edge_right_bottom");
        assert_eq!(pos5.x, 710.0);
        assert_eq!(pos5.y, 540.0);

        // edge_pct: 800 - 80 - 40 = 680.0; 600 - 120 - 20 = 460.0
        let pos6 = elem.get_position(&style, "edge_pct");
        assert_eq!(pos6.x, 680.0);
        assert_eq!(pos6.y, 460.0);
    }

    #[test]
    fn test_texture_element_lifecycle() {
        let tex = TextureElement::new("test.png");
        assert_eq!(tex.path, "test.png");

        let loaded = TextureElement::load_from_file("player.png");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().path, "player.png");

        let empty = TextureElement::load_from_file("");
        assert!(empty.is_none());

        let invalid = TextureElement::invalid();
        assert_eq!(invalid.path, "");

        let style = Style::new();
        let size = invalid.measure(&style, "missing");
        assert_eq!(size.x, 0.0);
        assert_eq!(size.y, 0.0);
    }

    #[test]
    fn test_render_texture_element_methods() {
        let target1 = RenderTexture {
            id: 1,
            texture: Texture {
                id: 1,
                width: 64,
                height: 64,
                mipmaps: 1,
                format: 7,
            },
            depth: unsafe { std::mem::zeroed() },
        };
        let target2 = RenderTexture {
            id: 2,
            texture: Texture {
                id: 2,
                width: 128,
                height: 128,
                mipmaps: 1,
                format: 7,
            },
            depth: unsafe { std::mem::zeroed() },
        };

        let mut elem = RenderTextureElement::new(target1);
        assert_eq!(elem.target().id, 1);
        assert_eq!(elem.target_mut().id, 1);

        elem.set_target(target2);
        assert_eq!(elem.target().id, 2);

        // Unload when window is not ready is a safe no-op
        elem.unload();
    }

    #[test]
    fn test_button_interaction_and_response() {
        set_viewport(800.0, 600.0, 800.0, 600.0);
        let style = parse_str(
            r#"
            btn = {
                left = 100,
                top = 50,
                width = 120,
                height = 40,
            },
            "#,
        )
        .unwrap();

        let child = MockElement::new(50.0, 20.0);
        let btn = Button::from_element(child);
        let resp = btn.place(&style, "btn");
        assert_eq!(resp.rect.x, 100.0);
        assert_eq!(resp.rect.y, 50.0);
        assert_eq!(resp.rect.width, 120.0);
        assert_eq!(resp.rect.height, 40.0);
        assert_eq!(resp.clicked(), false);
        assert_eq!(btn.clicked(), false);

        let btn_disabled = Button::new("Disabled").with_disabled(true);
        assert!(!btn_disabled.clicked_or_key(KeyboardKey::KEY_SPACE));
    }

    #[test]
    fn test_button_in_stack_layout_interaction() {
        set_viewport(800.0, 600.0, 800.0, 600.0);
        let style = parse_str(
            r#"
            menu = {
                left = 50,
                top = 50,
                direction = "vertical",
                gap = 10,
            },
            btn1 = { width = 100, height = 30 },
            btn2 = { width = 100, height = 30 },
            "#,
        )
        .unwrap();

        let b1 = Button::new("One");
        let b2 = Button::new("Two");

        let children: &[(&str, &dyn Element)] = &[("btn1", &b1), ("btn2", &b2)];
        let stack = StackLayout::new(children);
        stack.place(&style, "menu");

        // Verify that buttons in stack layout had their rects calculated during draw
        assert_eq!(b1.rect().x, 50.0);
        assert_eq!(b1.rect().y, 50.0);
        assert_eq!(b1.rect().width, 100.0);
        assert_eq!(b1.rect().height, 30.0);

        assert_eq!(b2.rect().x, 50.0);
        assert_eq!(b2.rect().y, 90.0); // 50 + 30 + 10 gap
        assert_eq!(b2.rect().width, 100.0);
        assert_eq!(b2.rect().height, 30.0);

        // Neither button was clicked with no mouse press
        assert!(!b1.clicked());
        assert!(!b2.clicked());
    }

    #[test]
    fn test_layout_children_accessors() {
        let child1 = MockElement::new(10.0, 10.0);
        let child2 = MockElement::new(20.0, 20.0);
        let children: &[(&str, &dyn Element)] = &[("c1", &child1), ("c2", &child2)];

        let stack = StackLayout::new(children);
        assert_eq!(stack.children().len(), 2);
        assert_eq!(stack.children()[0].0, "c1");
        assert_eq!(stack.children()[1].0, "c2");

        let grid = GridLayout::new(children);
        assert_eq!(grid.children().len(), 2);
        assert_eq!(grid.children()[0].0, "c1");
        assert_eq!(grid.children()[1].0, "c2");
    }
}
