//! # Input Action & Axis Mapping
//!
//! Abstracts physical hardware inputs (keyboard keys, mouse buttons) into semantic actions and axes.

use std::collections::HashMap;
use muxui::raylib::*;
use muxui::*;

/// Action and axis mapper for keyboard and mouse interactions.
#[derive(Debug, Clone, Default)]
pub struct Input {
    actions: HashMap<String, Vec<KeyboardKey>>,
    mouse_actions: HashMap<String, Vec<MouseButton>>,
    axes: HashMap<String, (KeyboardKey, KeyboardKey)>,
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

    /// Registers a named action mapped to one or more mouse buttons.
    pub fn register_mouse_action(&mut self, name: impl Into<String>, buttons: impl IntoIterator<Item = MouseButton>) {
        self.mouse_actions.insert(name.into(), buttons.into_iter().collect());
    }

    /// Registers a 1D virtual axis mapped to a positive key (+1.0) and a negative key (-1.0).
    pub fn register_axis(&mut self, name: impl Into<String>, positive: KeyboardKey, negative: KeyboardKey) {
        self.axes.insert(name.into(), (positive, negative));
    }

    /// Returns `true` if any key or button mapped to `action` is currently held down.
    pub fn is_action_down(&self, action: &str) -> bool {
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

    /// Computes the normalized axis value in the range `[-1.0, 1.0]`.
    ///
    /// Returns `1.0` if only the positive key is held down, `-1.0` if only the negative key is held,
    /// and `0.0` if both or neither are held down.
    pub fn get_axis(&self, name: &str) -> f32 {
        if let Some(&(pos, neg)) = self.axes.get(name) {
            let mut val = 0.0;
            if is_key_down(pos) {
                val += 1.0;
            }
            if is_key_down(neg) {
                val -= 1.0;
            }
            val
        } else {
            0.0
        }
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
        input.register_mouse_action("fire", [MOUSE_BUTTON_LEFT]);
        input.register_axis("horizontal", KEY_D, KEY_A);

        assert!(input.actions.contains_key("jump"));
        assert!(input.mouse_actions.contains_key("fire"));
        assert!(input.axes.contains_key("horizontal"));

        // In headless mode without key inputs, axis defaults to 0.0
        assert_eq!(input.get_axis("horizontal"), 0.0);
        assert_eq!(input.get_axis("nonexistent"), 0.0);
        assert!(!input.is_action_down("jump"));
        assert!(!input.is_action_pressed("jump"));
    }
}
