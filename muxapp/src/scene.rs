//! # Scene Management and Lifecycle Router
//!
//! Provides a first-class [`Scene`] trait and [`SceneRouter`] for managing multi-scene applications,
//! game states, and stack-based modal overlays (e.g. pause menus).

use std::collections::HashMap;
use muxui::Style;
use crate::input::Input;

/// Actions that a scene can return from [`Scene::update`] or [`Scene::draw_2d`] to command scene transitions.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SceneAction {
    /// No scene transition; continue running active scene.
    #[default]
    None,
    /// Switch active scene to the named scene, clearing the current stack.
    Switch(String),
    /// Push the named scene onto the stack as an overlay/modal.
    Push(String),
    /// Pop the current scene from the stack, restoring the previous scene.
    Pop,
}

/// Specifies how a scene interacts with underlying scenes on the [`SceneRouter`] stack during rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SceneLayerMode {
    /// Standalone opaque scene: only this scene is rendered when at the top of the stack (default).
    #[default]
    Opaque,
    /// Composite overlay scene: underlying scene(s) on the stack are rendered first (frozen, with UI input disabled),
    /// and this overlay scene is rendered on top without clearing the canvas.
    Overlay,
}

/// Trait defining the lifecycle and rendering passes of an application scene.
pub trait Scene<Context> {
    /// Specifies whether this scene is [`SceneLayerMode::Opaque`] or [`SceneLayerMode::Overlay`].
    fn layer_mode(&self) -> SceneLayerMode {
        SceneLayerMode::Opaque
    }

    /// Called when the scene becomes active or is entered.
    fn on_enter(&mut self, _ctx: &mut Context, _style: &Style) {}

    /// Called every frame to update scene logic and process input.
    ///
    /// Can optionally return a [`SceneAction`] to request switching, pushing, or popping scenes.
    fn update(&mut self, ctx: &mut Context, input: &Input, style: &Style, dt: f32) -> SceneAction {
        let _ = (ctx, input, style, dt);
        SceneAction::None
    }

    /// Called during the 3D rendering pass.
    fn draw_3d(&mut self, _ctx: &mut Context) {}

    /// Called during the 2D rendering pass to lay out and render user interfaces.
    ///
    /// Can return a [`SceneAction`] directly when UI buttons are clicked, eliminating the need
    /// for 1-frame deferred flag fields.
    fn draw_2d(&mut self, _ctx: &mut Context, _style: &Style) -> SceneAction {
        SceneAction::None
    }

    /// Called when the scene is deactivated, switched away from, or popped.
    fn on_exit(&mut self, _ctx: &mut Context) {}
}

/// A router and state machine managing registered scenes and the active scene stack.
pub struct SceneRouter<Context> {
    scenes: HashMap<String, Box<dyn Scene<Context>>>,
    stack: Vec<String>,
}

impl<Context> Default for SceneRouter<Context> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Context> SceneRouter<Context> {
    /// Creates a new empty [`SceneRouter`].
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            stack: Vec::new(),
        }
    }

    /// Registers a scene with a unique identifier.
    pub fn register(&mut self, name: impl Into<String>, scene: impl Scene<Context> + 'static) {
        self.scenes.insert(name.into(), Box::new(scene));
    }

    /// Returns `true` if a scene with the specified name has been registered.
    pub fn has_scene(&self, name: &str) -> bool {
        self.scenes.contains_key(name)
    }

    /// Switches the active scene, exiting any currently active scenes and replacing the stack with `name`.
    ///
    /// Calls `on_exit` on the current scene (if any) and `on_enter` on the new scene.
    /// Returns `true` if the scene was found and activated.
    pub fn switch(&mut self, name: &str, ctx: &mut Context, style: &Style) -> bool {
        if !self.scenes.contains_key(name) {
            return false;
        }

        // Call on_exit on currently active scene
        if let Some(active_id) = self.stack.last()
            && let Some(active_scene) = self.scenes.get_mut(active_id)
        {
            active_scene.on_exit(ctx);
        }

        self.stack.clear();
        self.stack.push(name.to_string());

        if let Some(new_scene) = self.scenes.get_mut(name) {
            new_scene.on_enter(ctx, style);
        }
        true
    }

    /// Pushes a new scene onto the stack without popping the current scene.
    ///
    /// Ideal for pause menus, inventories, or dialogue overlays.
    /// Calls `on_enter` on the newly pushed scene.
    pub fn push(&mut self, name: &str, ctx: &mut Context, style: &Style) -> bool {
        if !self.scenes.contains_key(name) {
            return false;
        }

        self.stack.push(name.to_string());
        if let Some(new_scene) = self.scenes.get_mut(name) {
            new_scene.on_enter(ctx, style);
        }
        true
    }

    /// Pops the active scene from the stack, calling `on_exit` on it.
    ///
    /// Returns the name of the popped scene, or `None` if the stack was already empty.
    pub fn pop(&mut self, ctx: &mut Context) -> Option<String> {
        if let Some(popped_name) = self.stack.pop() {
            if let Some(popped_scene) = self.scenes.get_mut(&popped_name) {
                popped_scene.on_exit(ctx);
            }
            Some(popped_name)
        } else {
            None
        }
    }

    /// Dispatches a [`SceneAction`] (Switch, Push, Pop, or None).
    pub fn dispatch_action(&mut self, action: SceneAction, ctx: &mut Context, style: &Style) {
        match action {
            SceneAction::Switch(target_name) => {
                self.switch(&target_name, ctx, style);
            }
            SceneAction::Push(target_name) => {
                self.push(&target_name, ctx, style);
            }
            SceneAction::Pop => {
                self.pop(ctx);
            }
            SceneAction::None => {}
        }
    }

    /// Executes a complete frame step for the scene stack:
    /// 1. Updates the top active scene.
    /// 2. Determines the composite render chain if the top scene is [`SceneLayerMode::Overlay`].
    /// 3. Renders underlying scene(s) with UI hit-testing disabled, followed by the active top scene.
    /// 4. Dispatches any [`SceneAction`] returned from `draw_2d` or `update`.
    pub fn step(&mut self, ctx: &mut Context, input: &Input, style: &Style, dt: f32) -> bool {
        if self.stack.is_empty() {
            return false;
        }

        let action_update = if let Some(active) = self.active_scene_mut() {
            active.update(ctx, input, style, dt)
        } else {
            return false;
        };

        // Find the lowest scene index in the stack that needs to be drawn
        let top_idx = self.stack.len() - 1;
        let mut base_idx = top_idx;
        while base_idx > 0 {
            let name = &self.stack[base_idx];
            let mode = self
                .scenes
                .get(name)
                .map(|s| s.layer_mode())
                .unwrap_or(SceneLayerMode::Opaque);
            if mode == SceneLayerMode::Overlay {
                base_idx -= 1;
            } else {
                break;
            }
        }

        let mut action_draw = SceneAction::None;
        for idx in base_idx..=top_idx {
            let is_top = idx == top_idx;
            muxui::set_ui_input_enabled(is_top);
            let scene_name = self.stack[idx].clone();
            if let Some(scene) = self.scenes.get_mut(&scene_name) {
                scene.draw_3d(ctx);
                let act = scene.draw_2d(ctx, style);
                if is_top {
                    action_draw = act;
                }
            }
        }
        muxui::set_ui_input_enabled(true);

        let final_action = match action_draw {
            SceneAction::None => action_update,
            act => act,
        };
        self.dispatch_action(final_action, ctx, style);
        true
    }

    /// Returns the name of the currently active scene, if any.
    pub fn active_name(&self) -> Option<&str> {
        self.stack.last().map(|s| s.as_str())
    }

    /// Returns a mutable reference to the currently active scene.
    pub fn active_scene_mut(&mut self) -> Option<&mut Box<dyn Scene<Context>>> {
        let active_id = self.stack.last()?;
        self.scenes.get_mut(active_id)
    }

    /// Returns the number of scenes currently on the stack.
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// Returns `true` if there are no active scenes on the stack.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockScene {
        entered: bool,
        exited: bool,
        updates: i32,
        layer: SceneLayerMode,
        draw_action: SceneAction,
    }

    impl MockScene {
        fn new() -> Self {
            Self {
                entered: false,
                exited: false,
                updates: 0,
                layer: SceneLayerMode::Opaque,
                draw_action: SceneAction::None,
            }
        }

        fn overlay(draw_action: SceneAction) -> Self {
            Self {
                entered: false,
                exited: false,
                updates: 0,
                layer: SceneLayerMode::Overlay,
                draw_action,
            }
        }
    }

    impl Scene<i32> for MockScene {
        fn layer_mode(&self) -> SceneLayerMode {
            self.layer
        }

        fn on_enter(&mut self, ctx: &mut i32, _style: &Style) {
            self.entered = true;
            *ctx += 10;
        }

        fn update(&mut self, ctx: &mut i32, _input: &Input, _style: &Style, _dt: f32) -> SceneAction {
            self.updates += 1;
            *ctx += 1;
            SceneAction::None
        }

        fn draw_2d(&mut self, ctx: &mut i32, _style: &Style) -> SceneAction {
            *ctx += 5;
            self.draw_action.clone()
        }

        fn on_exit(&mut self, ctx: &mut i32) {
            self.exited = true;
            *ctx -= 10;
        }
    }

    #[test]
    fn test_scene_router_lifecycle_and_overlay_composition() {
        let mut router: SceneRouter<i32> = SceneRouter::new();
        router.register("menu", MockScene::new());
        router.register("pause_overlay", MockScene::overlay(SceneAction::Pop));

        let mut context = 0;
        let style = Style::new();
        let input = Input::new();

        // Switch to menu
        assert!(router.switch("menu", &mut context, &style));
        assert_eq!(router.active_name(), Some("menu"));
        assert_eq!(context, 10); // entered menu (+10)

        // Step active scene (menu): update (+1) + draw_2d (+5) = 16
        assert!(router.step(&mut context, &input, &style, 0.016));
        assert_eq!(context, 16);

        // Push overlay scene (+10 on_enter -> 26)
        assert!(router.push("pause_overlay", &mut context, &style));
        assert_eq!(router.active_name(), Some("pause_overlay"));
        assert_eq!(router.stack_depth(), 2);
        assert_eq!(context, 26);

        // Step with overlay active:
        // 1. pause_overlay.update (+1 -> 27)
        // 2. menu.draw_2d (+5 -> 32) because pause_overlay is SceneLayerMode::Overlay!
        // 3. pause_overlay.draw_2d (+5 -> 37) and returns SceneAction::Pop!
        // 4. Pop is dispatched immediately in the same frame (-10 on_exit -> 27)!
        assert!(router.step(&mut context, &input, &style, 0.016));
        assert_eq!(router.active_name(), Some("menu"));
        assert_eq!(router.stack_depth(), 1);
        assert_eq!(context, 27);

        // Pop menu
        assert_eq!(router.pop(&mut context), Some("menu".to_string()));
        assert_eq!(router.active_name(), None);
        assert!(router.is_empty());
        assert_eq!(context, 17); // 27 - 10 (exited menu)
    }
}
