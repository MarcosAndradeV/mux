# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

