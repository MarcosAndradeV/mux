//! # Input Action & Axis Mapping
//!
//! Abstracts physical hardware inputs (keyboard keys, mouse buttons) into semantic actions and axes.

use std::collections::{HashMap, HashSet};
use muxui::raylib::*;
use muxui::*;

/// Continuous hardware mouse movement or wheel axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseAxis {
    /// Horizontal mouse movement delta (positive = right).
    X,
    /// Vertical mouse movement delta (positive = down).
    Y,
    /// Mouse scroll wheel movement.
    Wheel,
}

/// Action and axis mapper for keyboard and mouse interactions.
#[derive(Debug, Clone, Default)]
pub struct Input {
    actions: HashMap<String, Vec<KeyboardKey>>,
    mouse_actions: HashMap<String, Vec<MouseButton>>,
    axes: HashMap<String, (KeyboardKey, KeyboardKey)>,
    mouse_axes: HashMap<String, (MouseAxis, f32)>,
    unmasked_actions: HashSet<String>,
    masked: bool,
}

impl Input {
    /// Creates a new empty [`Input`] mapper.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a named action mapped to one or more keyboard keys.
    pub fn register_action(&mut self, name: impl Into<String>, keys: impl IntoIterator<Item = KeyboardKey>) {
        self.actions.insert(name.into(), keys.into_iter().collect());
    }

    /// Registers a named UI/system action mapped to keyboard keys that remains active even when gameplay input is masked.
    pub fn register_ui_action(&mut self, name: impl Into<String>, keys: impl IntoIterator<Item = KeyboardKey>) {
        let name_str = name.into();
        self.unmasked_actions.insert(name_str.clone());
        self.actions.insert(name_str, keys.into_iter().collect());
    }

    /// Marks an existing action name as allowed even when gameplay input is masked.
    pub fn allow_when_masked(&mut self, name: impl Into<String>) {
        self.unmasked_actions.insert(name.into());
    }

    /// Sets whether gameplay actions and axes are currently masked (e.g. during modal dialogs or text input).
    pub fn set_masked(&mut self, masked: bool) {
        self.masked = masked;
    }

    /// Returns `true` if gameplay input is currently masked.
    pub fn is_masked(&self) -> bool {
        self.masked
    }

    /// Registers a named action mapped to one or more mouse buttons.
    pub fn register_mouse_action(&mut self, name: impl Into<String>, buttons: impl IntoIterator<Item = MouseButton>) {
        self.mouse_actions.insert(name.into(), buttons.into_iter().collect());
    }

    /// Registers a 1D virtual axis mapped to a positive key (+1.0) and a negative key (-1.0).
    pub fn register_axis(&mut self, name: impl Into<String>, positive: KeyboardKey, negative: KeyboardKey) {
        self.axes.insert(name.into(), (positive, negative));
    }

    /// Registers a continuous mouse delta axis with a sensitivity multiplier.
    pub fn register_mouse_axis(&mut self, name: impl Into<String>, axis: MouseAxis, sensitivity: f32) {
        self.mouse_axes.insert(name.into(), (axis, sensitivity));
    }

    /// Returns `true` if any key or button mapped to `action` is currently held down.
    pub fn is_action_down(&self, action: &str) -> bool {
        if self.masked && !self.unmasked_actions.contains(action) {
            return false;
        }
        if let Some(keys) = self.actions.get(action) {
            for &key in keys {
                if is_key_down(key) {
                    return true;
                }
            }
        }
        if let Some(buttons) = self.mouse_actions.get(action) {
            for &btn in buttons {
                if is_mouse_button_down(btn) {
                    return true;
                }
            }
        }
        false
    }

    /// Returns `true` if any key or button mapped to `action` was pressed in the current frame.
    pub fn is_action_pressed(&self, action: &str) -> bool {
        if self.masked && !self.unmasked_actions.contains(action) {
            return false;
        }
        if let Some(keys) = self.actions.get(action) {
            for &key in keys {
                if is_key_pressed(key) {
                    return true;
                }
            }
        }
        if let Some(buttons) = self.mouse_actions.get(action) {
            for &btn in buttons {
                if is_mouse_button_pressed(btn) {
                    return true;
                }
            }
        }
        false
    }

    /// Computes the axis value, combining keyboard input and mouse delta if registered.
    ///
    /// - Returns `0.0` if gameplay input is currently masked via [`Input::set_masked`].
    /// - Keyboard contributions range between `[-1.0, 1.0]`.
    /// - Mouse contributions scale virtual delta/wheel by the configured sensitivity.
    pub fn get_axis(&self, name: &str) -> f32 {
        if self.masked {
            return 0.0;
        }
        let mut val = 0.0;
        if let Some(&(pos, neg)) = self.axes.get(name) {
            if is_key_down(pos) {
                val += 1.0;
            }
            if is_key_down(neg) {
                val -= 1.0;
            }
        }
        if let Some(&(axis, sensitivity)) = self.mouse_axes.get(name) {
            let delta = self.mouse_delta();
            let mouse_val = match axis {
                MouseAxis::X => delta.x * sensitivity,
                MouseAxis::Y => delta.y * sensitivity,
                MouseAxis::Wheel => get_mouse_wheel_move() * sensitivity,
            };
            val += mouse_val;
        }
        val
    }

    /// Returns the mouse movement delta scaled to virtual canvas coordinates.
    pub fn mouse_delta(&self) -> Vector2 {
        get_virtual_mouse_delta()
    }

    /// Returns the mouse position in virtual canvas coordinates.
    pub fn mouse_position(&self) -> Vector2 {
        get_virtual_mouse_position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_registration_and_axes() {
        let mut input = Input::new();
        input.register_action("jump", [KEY_SPACE, KEY_W]);
        input.register_ui_action("pause", [KEY_ESCAPE, KEY_P]);
        input.register_mouse_action("fire", [MOUSE_BUTTON_LEFT]);
        input.register_axis("horizontal", KEY_D, KEY_A);
        input.register_mouse_axis("pitch", MouseAxis::Y, 0.05);

        assert!(input.actions.contains_key("jump"));
        assert!(input.actions.contains_key("pause"));
        assert!(input.unmasked_actions.contains("pause"));
        assert!(input.mouse_actions.contains_key("fire"));
        assert!(input.axes.contains_key("horizontal"));
        assert!(input.mouse_axes.contains_key("pitch"));

        // Test masking state
        assert!(!input.is_masked());
        input.set_masked(true);
        assert!(input.is_masked());
        assert_eq!(input.get_axis("horizontal"), 0.0);

        input.set_masked(false);
        assert_eq!(input.get_axis("horizontal"), 0.0);
        assert_eq!(input.get_axis("pitch"), 0.0);
        assert_eq!(input.get_axis("nonexistent"), 0.0);
        assert!(!input.is_action_down("jump"));
        assert!(!input.is_action_pressed("jump"));
    }
}
