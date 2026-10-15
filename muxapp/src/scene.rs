//! # Scene Management and Lifecycle Router
//!
//! Provides a first-class [`Scene`] trait and [`SceneRouter`] for managing multi-scene applications,
//! game states, and stack-based modal overlays (e.g. pause menus).

use std::collections::HashMap;
use muxui::Style;
use crate::input::Input;

/// Actions that a scene can return from [`Scene::update`] to command scene transitions.
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

/// Trait defining the lifecycle and rendering passes of an application scene.
pub trait Scene<Context> {
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
    fn draw_3d(&mut self, _ctx: &Context) {}

    /// Called during the 2D rendering pass to lay out and render user interfaces.
    fn draw_2d(&mut self, _ctx: &Context, _style: &Style) {}

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
    }

    impl MockScene {
        fn new() -> Self {
            Self {
                entered: false,
                exited: false,
                updates: 0,
            }
        }
    }

    impl Scene<i32> for MockScene {
        fn on_enter(&mut self, ctx: &mut i32, _style: &Style) {
            self.entered = true;
            *ctx += 10;
        }

        fn update(&mut self, ctx: &mut i32, _input: &Input, _style: &Style, _dt: f32) -> SceneAction {
            self.updates += 1;
            *ctx += 1;
            SceneAction::None
        }

        fn on_exit(&mut self, ctx: &mut i32) {
            self.exited = true;
            *ctx -= 10;
        }
    }

    #[test]
    fn test_scene_router_lifecycle() {
        let mut router: SceneRouter<i32> = SceneRouter::new();
        router.register("menu", MockScene::new());
        router.register("game", MockScene::new());

        let mut context = 0;
        let style = Style::new();
        let input = Input::new();

        // Switch to menu
        assert!(router.switch("menu", &mut context, &style));
        assert_eq!(router.active_name(), Some("menu"));
        assert_eq!(context, 10); // entered menu

        // Update active scene
        if let Some(scene) = router.active_scene_mut() {
            scene.update(&mut context, &input, &style, 0.016);
        }
        assert_eq!(context, 11);

        // Push game overlay
        assert!(router.push("game", &mut context, &style));
        assert_eq!(router.active_name(), Some("game"));
        assert_eq!(router.stack_depth(), 2);
        assert_eq!(context, 21); // entered game

        // Pop game overlay
        assert_eq!(router.pop(&mut context), Some("game".to_string()));
        assert_eq!(router.active_name(), Some("menu"));
        assert_eq!(router.stack_depth(), 1);
        assert_eq!(context, 11); // exited game

        // Pop menu
        assert_eq!(router.pop(&mut context), Some("menu".to_string()));
        assert_eq!(router.active_name(), None);
        assert!(router.is_empty());
        assert_eq!(context, 1); // exited menu
    }
}
