# Mux Developer Guide: Point-and-Click Architecture

Welcome to the **Mux** development guide! This document is a comprehensive handbook designed to help you build point-and-click games and interactive applications using the Mux ecosystem.

---

## 1. Crate Architecture

The Mux workspace is modularized into dedicated crates to ensure clean separation of concerns, easy unit testing, and component reusability.

```
                  +--------------------------------+
                  |           text-mux             |  (Game Binary Crate)
                  +--------------------------------+
                                  |
                                  v
                  +--------------------------------+
                  |            muxapp              |  (Application Shell & Target Loop)
                  +--------------------------------+
                     /                            \
                    v                              v
  +--------------------------------+      +--------------------------------+
  |            muxui               |      |          muxengine             |  (Pure Logic State VM)
  |  (UI elements & layout pipeline|      +--------------------------------+
  +--------------------------------+                      |
                    \                                     v
                     \                    +--------------------------------+
                      +------------------>|           muxutils             |  (GSS Helpers & Raylib FFI)
                                          +--------------------------------+
                                                          |
                                                          v
                                          +--------------------------------+
                                          |             GSS                |  (Graph Style Sheet Crate)
                                          +--------------------------------+
```

### Member Crate Overview
*   [`GSS`](file:///home/marcos/Projects/mux/GSS/src/lib.rs): Parsers and data models for Graph Style Sheets (nested objects, references, percentages, integers, and floats).
*   [`muxutils`](file:///home/marcos/Projects/mux/muxutils/src/lib.rs): General utilities, viewport calculations, scaling algorithms, and raw Raylib FFI bindings.
*   [`muxengine`](file:///home/marcos/Projects/mux/muxengine/src/lib.rs): Pure point-and-click game state logic, dialogue state structures, actionVM triggers, and condition evaluations. It has **no dependency** on graphics rendering, making it 100% unit-testable in headless mode.
*   [`muxui`](file:///home/marcos/Projects/mux/muxui/src/lib.rs): Standard layout components (`StackLayout`, `ButtonElement`, `DialogueElement`, `HotspotElement`, `InventoryElement`, etc.) implementing the [`Element`] trait.
*   [`muxapp`](file:///home/marcos/Projects/mux/muxapp/src/lib.rs): The windowing shell which runs Raylib, tracks F5 stylesheet reloading, manages aspect ratio target viewport resizing, and drives the cooperative update loop. It defines the [`Engine`] trait for pluggable engine backends and provides the optional `point-and-click` feature flag for built-in `muxengine` integration.


---

## 2. Decoupled State & Interaction Loop

Mux uses an in-process cooperative Client-Server loop. The logic and presentation layers interact by passing messages or reading state views.

```
  +------------------------+                     +------------------------+
  |       UI Client        |                     |      Game Engine       |
  |  - Listens for inputs  |                     |  - Manages GameState   |
  |  - Places UI elements  |                     |  - Runs GSS triggers   |
  |  - Emits EngineEvents  |                     |  - Emits SceneView     |
  +------------------------+                     +------------------------+
              |                                               ^
              |               EngineEvent                     |
              +-----------------------------------------------+
                                 (Sends action)
              
              <-----------------------------------------------+
                                 SceneView
                                 (Pushes snapshot)
```

1.  **EngineEvent**: Captured from user clicks/presses and processed by the engine:
    ```rust
    pub enum EngineEvent {
        ClickHotspot { hotspot_id: String },
        ClickUiButton { button_id: String },
        UseItem { item_id: String, target_hotspot_id: String },
        SelectDialogOption { option_index: usize },
        SkipDialogue,
    }
    ```
2.  **SceneView**: Constructed by the engine to describe what should render on screen:
    ```rust
    pub struct SceneView {
        pub scene_id: String,
        pub background_texture: String,
        pub hotspots: Vec<HotspotView>,
        pub dialogue: Option<DialogueView>,
        pub inventory: Vec<ItemView>,
    }
    ```

### Context wrapper
The application loop feeds a [`Context<AppState, E>`] structure to your update callbacks. This provides safe access to both your custom variables and the engine controller:
```rust
fn update(ctx: &mut Context<AppState, EngineController>, style: &Style) {
    let state = ctx.state_mut(); // Access custom state variables
    let engine = ctx.engine_mut(); // Process custom logical transitions
}
```
For engine-less apps, `E = ()` and `ctx.state_mut()` is used directly.


---

## 3. Designing Scenes in GSS

Instead of hardcoding positions and action coordinates in Rust, all scenes, backgrounds, hotspot shapes, and actions are configured inside GSS stylesheets (e.g. `data/test.gss`).

### Dynamic Backgrounds
Background properties can be configured as a raw string filepath (fullscreen default) or as a nested layout object:
```gss
scenes = {
    hallway = {
        // Nested background object (places texture at custom offset and scale)
        background = {
            path = "data/assets/hallway_bg.png",
            top = 10%,
            left = 10%,
            scale = 300%,
        }
    }
}
```

### Hotspot Configurations
Hotspots are interactive areas drawn on top of the background. They support a variety of logical properties:
```gss
hotspots = {
    door_to_kitchen = {
        left = 21%,         // Horizontal offset (percentage or absolute pixels)
        top = 25%,          // Vertical offset
        width = 120.0,      // Hotspot width
        height = 240.0,     // Hotspot height
        cursor = "exit_left", // Hover cursor style (walk, look, grab, talk, exit_left, exit_right)
        tooltip = "Kitchen Door",
        on_click = "action:change_scene kitchen", // Trigger action on click
    },
    brass_key = {
        left = 40%, top = 70%, width = 60.0, height = 40.0,
        cursor = "grab",
        tooltip = "Take key",
        // Conditional visibility evaluated against GameState flags
        visible_if = "!flags.has_brass_key",
        on_click = "action:pick_up_item brass_key",
    }
}
```

### Evaluatable Flags & Triggers
The engine evaluates visibility scripts using the prefix `flags.` (looks up boolean markers) or `inventory.` (checks if item is collected). Supported triggers:
*   `action:change_scene <scene_id>`: Transitions the engine state to the target scene.
*   `action:pick_up_item <item_id>`: Inserts item into inventory and automatically sets the boolean flag `has_<item_id>` to `true`.
*   `action:set_flag <flag_name> <true/false>`: Mutates game variables.
*   `action:show_dialogue <speaker> '<text_message>'`: Spawns a dialog box.
*   `script:<hook_name>`: Delegates to a custom Rust closure (registered in code) to handle complex branching paths.

---

## 4. Sizing, Elements, & Slicing

All drawable objects should implement the [`Element`] trait. Sizing and layout placement is handled automatically by the styling engine relative to the viewport.

### The Element Trait
```rust
pub trait Element {
    /// Renders the component on screen at the calculated coordinates.
    fn draw(&self, position: Vector2, style: &Style, name: &str);

    /// Measures the dimensions of this element under GSS.
    fn measure(&self, style: &Style, name: &str) -> Vector2;

    /// Checks if a mouse interaction click occurred on this element.
    fn event(&self) -> Event { Event::None }
}
```

### Responsive Scaling
Always draw elements using the `.place()` helper rather than drawing directly via coordinate logic. The place system resolves alignments (`align = "center"`, `valign = "middle"`) and scales percentage fields dynamically:
```rust
// Instantiates a layout element and positions it dynamically using layout rules:
let inv_el = InventoryElement::new(item_names);
inv_el.place(style, "inventory_panel");
```

---

## 5. Automated Texture DX

To remove the boilerplate of manual resource management, Mux uses a thread-local texture cache:

1.  **Lazy Loading**: The element (like `TextureElement`) simply stores a `String` path. During `.draw()` or `.measure()`, it requests the texture handle from the cache via `muxui::get_cached_texture(path)`.
2.  **No GPU Leaks**: Since textures are owned by the thread-local cache, individual elements do not need custom `Drop` hooks.
3.  **Automatic Cleanup**: The shell automatically invokes `muxui::clear_texture_cache()` inside `App::run` when window close procedures are invoked, freeing all loaded textures safely.

---

## 6. Walkthrough: Adding a New Scene

Here is a step-by-step example of how to add a new scene to your point-and-click game:

### Step 1: Update GSS Scene Config (`data/test.gss`)
Add a new object inside the `scenes` block:
```gss
scenes = {
    ...
    secret_room = {
        background = "data/assets/secret_room.png",
        hotspots = {
            chest = {
                left = 45%, top = 60%, width = 80.0, height = 50.0,
                cursor = "grab",
                tooltip = "Iron Chest",
                on_click = "script:open_chest", // Handled by Rust hook
            },
            exit_door = {
                left = 5%, top = 20%, width = 100.0, height = 220.0,
                cursor = "exit_left",
                tooltip = "Back to Kitchen",
                on_click = "action:change_scene kitchen",
            }
        }
    }
}
```

### Step 2: Register Script Hooks in Rust (`src/main.rs`)
In your initialization block, define the logic branch for the custom `open_chest` hook:
```rust
App::init(800, 600, "My Game", init_state)
    .set_script_hook(|state, script_name| {
        match script_name {
            "open_chest" => {
                if state.inventory.contains("brass_key") {
                    // Mutate state and unlock dialogue
                    state.flags.insert("chest_unlocked".to_string(), true);
                    Some("action:show_dialogue Narrative 'You unlocked the chest! You found a magic sword!'".to_string())
                } else {
                    Some("action:show_dialogue Narrative 'The lock is too strong. I need a key.'".to_string())
                }
            }
            _ => None,
        }
    })
    .run();
```

By separating game scripts, layouts, and logic triggers from your rendering and drawing pipelines, your point-and-click application remains clean, structured, and easy to maintain.
