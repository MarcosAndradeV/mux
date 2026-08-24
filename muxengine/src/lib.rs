#![warn(missing_docs)]
//! `muxengine` is the core point-and-click game engine for the Mux framework.
//!
//! It manages game state (`GameState`), cursor representations (`CursorType`),
//! event processing (`EngineEvent`), scene snapshot generation (`SceneView`),
//! and trigger/action execution (`EngineController`).

use muxutils::gss::{Gss, Object};
use std::collections::{HashMap, HashSet};

/// The cursor types for hotspot interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorType {
    /// Standard pointer cursor.
    Pointer,
    /// Walk indicator (e.g. arrow or boots).
    Walk,
    /// Eye symbol to examine things.
    Look,
    /// Hand symbol to grab/pickup items.
    Grab,
    /// Talk bubble for speaking.
    Talk,
    /// Transition arrow to exit left.
    ExitLeft,
    /// Transition arrow to exit right.
    ExitRight,
}

impl CursorType {
    /// Parse cursor string from GSS.
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "walk" => Self::Walk,
            "look" => Self::Look,
            "grab" | "take" => Self::Grab,
            "talk" | "speak" => Self::Talk,
            "exit_left" => Self::ExitLeft,
            "exit_right" => Self::ExitRight,
            _ => Self::Pointer,
        }
    }
}

/// Dialogue snapshot to display on the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueView {
    /// Name of the character speaking.
    pub speaker: String,
    /// The speech content.
    pub text: String,
    /// Multiple-choice options (if any).
    pub options: Vec<String>,
}

/// Item snapshot inside the player's inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemView {
    /// Uniquely identifies the item.
    pub id: String,
    /// Display name of the item.
    pub name: String,
    /// Texture asset filepath.
    pub icon_texture: String,
}

/// Visual hotspot region in the active scene.
#[derive(Debug, Clone)]
pub struct HotspotView {
    /// Unique identifier for this hotspot.
    pub id: String,
    /// Interaction cursor.
    pub hover_cursor: CursorType,
    /// Hover description text.
    pub tooltip: String,
}

impl PartialEq for HotspotView {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.hover_cursor == other.hover_cursor
            && self.tooltip == other.tooltip
    }
}

/// EngineEvent emitted by user interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    /// Sent when the player clicks on a hotspot area in the active scene.
    ClickHotspot {
        /// Unique identifier of the clicked hotspot.
        hotspot_id: String,
    },
    /// Sent when a UI control button is pressed.
    ClickUiButton {
        /// Unique identifier of the pressed UI button.
        button_id: String,
    },
    /// Sent when the player uses an item on a hotspot.
    UseItem {
        /// Unique identifier of the item being used.
        item_id: String,
        /// Unique identifier of the target hotspot area.
        target_hotspot_id: String,
    },
    /// Sent when selecting a dialogue branch option.
    SelectDialogOption {
        /// Index of the chosen dialogue option (0-based).
        option_index: usize,
    },
    /// Sent when clicking screen/dialogue to skip to the next line.
    SkipDialogue,
}

/// Snapshot of the visible game scene.
#[derive(Debug, Clone)]
pub struct SceneView {
    /// Unique scene name.
    pub scene_id: String,
    /// Filepath or asset descriptor of the background.
    pub background_texture: String,
    /// List of interactive hotspots.
    pub hotspots: Vec<HotspotView>,
    /// Dialogue overlay status.
    pub dialogue: Option<DialogueView>,
    /// Current inventory.
    pub inventory: Vec<ItemView>,
}
impl SceneView {
    /// Retrieve the GSS styling object for hotspots in the current scene.
    pub fn get_hotspots_style<'a>(&self, base: &'a Gss) -> Option<&'a Object> {
        base.get(&["scenes", &self.scene_id, "hotspots"])
    }
}

impl PartialEq for SceneView {
    fn eq(&self, other: &Self) -> bool {
        self.scene_id == other.scene_id
            && self.background_texture == other.background_texture
            && self.hotspots == other.hotspots
            && self.dialogue == other.dialogue
            && self.inventory == other.inventory
    }
}

/// The state variables, flags, and inventory of the game.
#[derive(Debug, Clone, Default)]
pub struct GameState {
    /// Boolean flags indicating game progress.
    pub flags: HashMap<String, bool>,
    /// Numeric state variables.
    pub variables: HashMap<String, i32>,
    /// Set of collected item IDs.
    pub inventory: HashSet<String>,
}

/// Custom script hooks for external engine logic.
pub type ScriptHook = Box<dyn Fn(&mut GameState, &str) -> Option<String>>;

/// The core point-and-click engine controller.
pub struct EngineController {
    current_scene: String,
    state: GameState,
    active_dialogue: Option<DialogueView>,
    script_hook: Option<ScriptHook>,
}

impl EngineController {
    /// Create a new point-and-click engine context.
    pub fn new(initial_scene: String) -> Self {
        Self {
            current_scene: initial_scene,
            state: GameState::default(),
            active_dialogue: None,
            script_hook: None,
        }
    }

    /// Set an optional hook to run custom script triggers.
    pub fn set_script_hook<F>(&mut self, hook: F)
    where
        F: Fn(&mut GameState, &str) -> Option<String> + 'static,
    {
        self.script_hook = Some(Box::new(hook));
    }

    /// Get a mutable reference to the game state.
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Retrieve the current scene ID.
    pub fn current_scene_id(&self) -> &str {
        &self.current_scene
    }

    /// Query GSS and calculate the layout values to output a SceneView snapshot.
    pub fn current_view(&self, scenes: &Gss) -> SceneView {
        let background_texture = muxutils::get_string_field(
            scenes,
            &["scenes", &self.current_scene, "background"],
            "data/assets/fallback_bg.png",
        );

        let mut hotspots = Vec::new();

        if let Some(hotspots_obj) =
            scenes.get::<Object>(&["scenes", &self.current_scene, "hotspots"])
        {
            for hotspot_name in hotspots_obj.get_fields() {
                // Check if this hotspot should be visible based on its conditions
                let cond_path = [
                    "scenes",
                    &self.current_scene,
                    "hotspots",
                    hotspot_name,
                    "visible_if",
                ];
                if let Some(cond) = scenes.get::<String>(&cond_path) {
                    if !self.evaluate_condition(cond) {
                        continue;
                    }
                }

                let cursor_str = muxutils::get_string_field(
                    scenes,
                    &[
                        "scenes",
                        &self.current_scene,
                        "hotspots",
                        hotspot_name,
                        "cursor",
                    ],
                    "pointer",
                );

                let tooltip = muxutils::get_string_field(
                    scenes,
                    &[
                        "scenes",
                        &self.current_scene,
                        "hotspots",
                        hotspot_name,
                        "tooltip",
                    ],
                    hotspot_name,
                );

                hotspots.push(HotspotView {
                    id: hotspot_name.clone(),
                    hover_cursor: CursorType::from_str(&cursor_str),
                    tooltip,
                });
            }
        }

        // Build inventory views
        let inventory = self
            .state
            .inventory
            .iter()
            .map(|item_id| {
                let name = item_id
                    .replace('_', " ")
                    .chars()
                    .enumerate()
                    .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                    .collect::<String>();
                ItemView {
                    id: item_id.clone(),
                    name,
                    icon_texture: format!("data/assets/items/{}.png", item_id),
                }
            })
            .collect();

        SceneView {
            scene_id: self.current_scene.clone(),
            background_texture,
            hotspots,
            dialogue: self.active_dialogue.clone(),
            inventory,
        }
    }

    /// Process a UI interaction event.
    pub fn process_event(&mut self, scenes: &Gss, event: EngineEvent) {
        match event {
            EngineEvent::ClickHotspot { hotspot_id } => {
                let path = [
                    "scenes",
                    &self.current_scene,
                    "hotspots",
                    &hotspot_id,
                    "on_click",
                ];
                if let Some(on_click) = scenes.get::<String>(&path) {
                    let on_click_val = on_click.clone();
                    self.execute_trigger(&on_click_val);
                }
            }
            EngineEvent::SkipDialogue => {
                self.active_dialogue = None;
            }
            EngineEvent::UseItem {
                item_id,
                target_hotspot_id,
            } => {
                let path = [
                    "scenes",
                    &self.current_scene,
                    "hotspots",
                    &target_hotspot_id,
                    "on_use",
                ];
                if let Some(on_use_obj) = scenes.get::<Object>(&path) {
                    if let Some(action) = on_use_obj.get::<String>(&[&item_id]) {
                        let action_val = action.clone();
                        self.execute_trigger(&action_val);
                    }
                }
            }
            _ => {}
        }
    }

    /// Evaluate a conditional expression string against current state.
    fn evaluate_condition(&self, cond: &str) -> bool {
        let cond = cond.trim();
        if cond.is_empty() {
            return true;
        }
        let invert = cond.starts_with('!');
        let key = if invert { &cond[1..] } else { cond };

        let val = if let Some(stripped) = key.strip_prefix("flags.") {
            *self.state.flags.get(stripped).unwrap_or(&false)
        } else if let Some(stripped) = key.strip_prefix("inventory.") {
            self.state.inventory.contains(stripped)
        } else {
            *self.state.flags.get(key).unwrap_or(&false)
        };

        if invert { !val } else { val }
    }

    /// Execute a trigger string, dispatching to actions or script hooks.
    fn execute_trigger(&mut self, trigger: &str) {
        let trigger = trigger.trim();
        if let Some(action_str) = trigger.strip_prefix("action:") {
            self.execute_action(action_str);
        } else if let Some(script_name) = trigger.strip_prefix("script:") {
            if let Some(hook) = &self.script_hook {
                if let Some(resolved_action) = hook(&mut self.state, script_name) {
                    self.execute_trigger(&resolved_action);
                }
            }
        }
    }

    /// Parse and execute an engine action string.
    fn execute_action(&mut self, action_str: &str) {
        let action_str = action_str.trim();
        let parts: Vec<&str> = action_str.splitn(2, ' ').collect();
        if parts.is_empty() {
            return;
        }
        let command = parts[0];
        let arg = if parts.len() > 1 { parts[1].trim() } else { "" };

        match command {
            "change_scene" => {
                if !arg.is_empty() {
                    self.current_scene = arg.to_string();
                }
            }
            "pick_up_item" => {
                if !arg.is_empty() {
                    self.state.inventory.insert(arg.to_string());
                    self.state.flags.insert(format!("has_{}", arg), true);
                }
            }
            "set_flag" => {
                let sub_parts: Vec<&str> = arg.splitn(2, ' ').collect();
                if sub_parts.len() == 2 {
                    let flag_name = sub_parts[0].trim().to_string();
                    let val = sub_parts[1].trim() == "true";
                    self.state.flags.insert(flag_name, val);
                }
            }
            "show_dialogue" => {
                let sub_parts: Vec<&str> = arg.splitn(2, ' ').collect();
                if sub_parts.len() == 2 {
                    let speaker = sub_parts[0].trim().to_string();
                    let text = sub_parts[1]
                        .trim()
                        .trim_matches('\'')
                        .trim_matches('"')
                        .to_string();
                    self.active_dialogue = Some(DialogueView {
                        speaker,
                        text,
                        options: Vec::new(),
                    });
                }
            }
            "clear_dialogue" => {
                self.active_dialogue = None;
            }
            _ => {
                muxutils::raylib::log_warn!("MUXENGINE: Unknown action command: \"{}\"", command);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_evaluation() {
        let mut controller = EngineController::new("scene_1".to_string());

        controller.state.flags.insert("has_item".to_string(), true);
        controller.state.flags.insert("unlocked".to_string(), false);
        controller.state.inventory.insert("brass_key".to_string());

        assert!(controller.evaluate_condition("flags.has_item"));
        assert!(!controller.evaluate_condition("flags.unlocked"));
        assert!(!controller.evaluate_condition("!flags.has_item"));
        assert!(controller.evaluate_condition("!flags.unlocked"));

        assert!(controller.evaluate_condition("inventory.brass_key"));
        assert!(!controller.evaluate_condition("inventory.wooden_key"));
    }

    #[test]
    fn test_execute_actions() {
        let mut controller = EngineController::new("scene_1".to_string());

        controller.execute_action("pick_up_item shiny_gem");
        assert!(controller.state.inventory.contains("shiny_gem"));
        assert_eq!(controller.state.flags.get("has_shiny_gem"), Some(&true));

        controller.execute_action("set_flag door_open true");
        assert_eq!(controller.state.flags.get("door_open"), Some(&true));

        controller.execute_action("change_scene scene_2");
        assert_eq!(controller.current_scene, "scene_2");

        controller.execute_action("show_dialogue Hero 'Welcome home!'");
        assert_eq!(
            controller.active_dialogue,
            Some(DialogueView {
                speaker: "Hero".to_string(),
                text: "Welcome home!".to_string(),
                options: Vec::new(),
            })
        );
    }
}
