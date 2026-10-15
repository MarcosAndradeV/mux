# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-10-15

### Added
- **3D Render Batch Limit Management (`muxui::d3`):**
  - Safe chunked mesh rendering via `d3.draw_trimesh_buffered(&vertices)` with automatic `rlCheckRenderBatchLimit` invocation before chunk drawing to prevent Raylib buffer overflows on procedural meshes (>8,192 vertices).
  - `Vertex3D` struct with `position`, `color`, and optional `normal` and `uv` coordinates.
  - Convenience helper `d3.draw_triangles(&positions, &colors)`.
  - 3D billboard helper `d3.draw_billboard(texture, position, size, tint, &camera)`.
- **Continuous Mouse Delta Axis Mapping (`muxapp`):**
  - `MouseAxis` enum supporting horizontal (`X`), vertical (`Y`), and mouse scroll (`Wheel`) axes.
  - `Input::register_mouse_axis(name, axis, sensitivity)` for continuous analog input.
  - Composite `Input::get_axis(name)` seamlessly summing keyboard discrete contributions and virtual mouse delta/wheel movements.
  - Builder method `App::register_mouse_axis(name, axis, sensitivity)`.
- **Safe Text FFI & 3D Screen Projection (`raylib-rs` / `muxutils`):**
  - Upgraded `draw_text` and `measure_text` in `raylib-rs` to accept `impl AsRef<str>`, preventing FFI memory smearing and eliminating manual `CString` wrapping.
  - Added `draw_text_ex_str` and `measure_text_ex_str` for font-based rendering with `&str`.
  - Added `Color::lerp(&self, target, factor)` for linear color interpolation.
  - Added `get_virtual_world_to_screen(camera, position)` in `muxutils`, converting 3D coordinates to 2D virtual canvas positions while properly handling camera culling (returns `None` when behind the camera).
- **Dynamic GSS Element Classes (`muxui` / `muxutils`):**
  - Added `.with_class(class)` and `.set_class(class)` to `TextElement`, `Button`, and `RectangleElement`.
  - Hierarchical class-aware style resolution via `get_color_field_with_class`, `get_f32_field_with_class`, and `get_bool_field_with_class` (`element.class.property` -> `element_class.property` -> `class.property` -> `element.property`).
- **Interactive Verification Demo (`muxtest`):**
  - Upgraded demo to showcase procedural 12,000+ vertex terrain mesh, continuous mouse look, dynamic GSS alert badge toggling, and in-world 3D HUD waypoint tracking with color pulsing.

## [0.2.0] - 2026-10-14

### Added
- **Safe 3D Rendering RAII (`muxui::d3`):**
  - Safe RAII `draw_3d(camera, |d3| { ... })` wrapper eliminating manual `unsafe { BeginMode3D/EndMode3D }`.
  - Matrix stack scoping via `d3.push_matrix(|ctx| { ... })` and safe transformation helpers (`rotate`, `translate`, `scale`).
  - Safe 3D drawing primitives: `draw_cube`, `draw_cube_wires`, `draw_sphere`, `draw_sphere_wires`, `draw_cylinder`, `draw_plane`, `draw_line_3d`, `draw_grid`.
  - Fluent `Camera3DBuilder` for easy construction of `Camera3D` instances.
- **Native 3D & Vector Math (`raylib-rs`):**
  - Standard operator traits implemented directly on `Vector3`: `Add`, `Sub`, `Mul<Vector3>`, `Mul<f32>`, `Div<Vector3>`, `Div<f32>`, `Neg`.
  - Core vector methods on `Vector3`: `dot`, `cross`, `length`, `length_sq`, `normalize`, `distance`, `lerp`, `scale`.
  - Scalar and negation traits on `Vector2` (`Mul<f32>`, `Div<f32>`, `Neg`, `dot`, `distance`, `lerp`).
  - `Matrix::identity()`, `Matrix::from_basis(...)`, and matrix multiplication operator (`std::ops::Mul`).
  - Safe window and cursor control wrappers: `disable_cursor()`, `enable_cursor()`, `maximize_window()`.
- **First-Class Scene Management (`muxapp`):**
  - Standard `Scene<Context>` trait with `on_enter`, `update`, `draw_3d`, `draw_2d`, and `on_exit` lifecycle methods.
  - Stack-based `SceneRouter` supporting `switch`, `push`, and `pop` for modal overlays and pause menus.
  - Command-based `SceneAction` enum returned from `Scene::update` (`Switch`, `Push`, `Pop`, `None`).
  - `App::add_scene(name, scene)` and `App::set_active_scene(name)`.
- **Typed GSS Data Binding (`muxutils`):**
  - `FromGssValue` trait and `GssExt` extension trait for `gss::Object` (`get_val`, `get_val_or`).
  - Native deserialization for `Vector2`, `Vector3`, `Color`, `f32`, `u32`, `i32`, `usize`, `bool`, and `String`.
- **Input Action & Axis Mapping (`muxapp`):**
  - `Input` mapper with `register_action`, `register_mouse_action`, and `register_axis`.
  - Semantic queries: `is_action_down`, `is_action_pressed`, `get_axis`.
  - `App::on_update_with_input(|ctx, input, style, dt| { ... })`.
- **3D Viewport & Mouse Virtualization (`muxutils`):**
  - `get_virtual_mouse_delta()` scaling mouse delta by virtual canvas scale factor.
  - `get_virtual_screen_to_world_ray(camera)` raycasting from virtual canvas coordinates.

## [0.1.0-alpha] - 2026-10-13

### Added
- Stateless immediate `Button` API returning interaction `Response` (`clicked`, `hovered`, `pressed`, `rect`).
- Immediate interaction support for buttons inside layout containers (`StackLayout`, `GridLayout`).
- Top-level `muxui::button` convenience helper.
- Support for `active_color` / `pressed_color`, explicit GSS `width` / `height`, and padding on buttons.
- Edge anchoring support for `right` and `bottom` in `Element::get_position` with relative `%` and pixel scaling.
- Relative `%`-based sizing support for `RectangleElement`.
- `App::new(width, height, title)` for lightweight apps with unit context.
- Default `responsive = true` in application shell for dynamic window resizing.
- Unified stylesheet reloading across F5 key press and filesystem watch modifications.

### Removed
- Button context caching requirement (`cached_rec` and `new_with_no_cached_rec`).
- Point-and-click leftovers: `Event` enum, `Element::event`, and `UpdateElement`.
- `MOUSE_CLICK_RADIUS` in `muxutils`; buttons now perform exact point-in-rect hit testing.
- Unused `notify` dependency in `muxui/Cargo.toml`.

