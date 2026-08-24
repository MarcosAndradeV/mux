use muxapp::muxengine::*;
use muxapp::muxui::*;
use muxapp::App;

struct Context;

fn main() {
    App::init(800, 600, "Mux Point-and-Click Engine", init_context)
        .on_update(update)
        .on_reload(reload)
        .set_style_file("data/layout.gss")
        .set_initial_scene("scene_hallway")
        .set_fps(30)
        .run();
}

fn init_context() -> Context {
    Context
}

fn reload(_ctx: &mut Context, engine: &mut EngineController, _style: &Style) {
    engine.set_script_hook(|state, script_name| {
        match script_name {
            "interact_drawer" => {
                if *state.flags.get("has_brass_key").unwrap_or(&false) {
                    Some("action:show_dialogue Drawer 'You unlocked the drawer using the brass key! Inside you found a shiny gemstone! You Win!'".to_string())
                } else {
                    Some("action:show_dialogue Drawer 'The drawer is locked tightly. It seems to require a key.'".to_string())
                }
            }
            _ => None
        }
    });
}

fn update(_ctx: &mut Context, engine: &mut EngineController, style: &Style) {
    let screen_w = get_screen_width() as f32;
    let screen_h = get_screen_height() as f32;

    // 1. Fetch current scene snapshot from the engine
    let view = engine.current_view(style, screen_w, screen_h);

    // 2. Process Input and emit Engine Events
    let mouse_pos = get_mouse_position();
    let mut hovered_hotspot = None;

    if view.dialogue.is_some() {
        // Dialogue blocks scene interaction. Any click or space key skips/closes the dialogue.
        if is_mouse_button_pressed(MOUSE_BUTTON_LEFT) || is_key_pressed(KEY_SPACE) {
            engine.process_event(style, EngineEvent::SkipDialogue);
        }
    } else {
        // Find if mouse is hovering over any hotspot using HotspotElement boundary check
        if let Some(hotspots_style) = style.get::<Style>(&[&view.scene_id, "hotspots"]) {
            for hotspot in &view.hotspots {
                let el = HotspotElement::new(&hotspot.id);
                let bounds = el.get_rec(hotspots_style, &hotspot.id);
                if check_collision_point_rec(mouse_pos, bounds) {
                    hovered_hotspot = Some((hotspot, bounds));
                    break;
                }
            }
        }

        // Trigger action on left click
        if let Some((hotspot, _)) = hovered_hotspot {
            if is_mouse_button_pressed(MOUSE_BUTTON_LEFT) {
                engine.process_event(
                    style,
                    EngineEvent::ClickHotspot {
                        hotspot_id: hotspot.id.clone(),
                    },
                );
            }
        }
    }

    // 3. Render the Scene
    begin_drawing();

    // Render background (fallback colors based on active scene)
    if view.scene_id == "scene_hallway" {
        clear_background(get_color(0x28201CFF)); // Warm brown/grey for hallway
    } else if view.scene_id == "scene_kitchen" {
        clear_background(get_color(0x1C2830FF)); // Slate blue for kitchen
    } else {
        clear_background(get_color(0x181818FF)); // Default dark background
    }

    // Draw scene header text
    let header_text = format!(
        "Scene: {}",
        view.scene_id.replace("scene_", "").to_uppercase()
    );
    draw_text(cstr!(&header_text), 20, 20, 24, LIGHTGRAY);

    // Draw active hotspots using HotspotElement::place!
    if let Some(hotspots_style) = style.get::<Style>(&[&view.scene_id, "hotspots"]) {
        for hotspot in &view.hotspots {
            let el = HotspotElement::new(&hotspot.id);
            el.place(hotspots_style, &hotspot.id);

            // Draw hotspot label inside/above the bounds
            let label = format!("[{}]", hotspot.tooltip);
            let bounds = el.get_rec(hotspots_style, &hotspot.id);
            let is_hovered = hovered_hotspot.map_or(false, |(h, _)| h.id == hotspot.id);

            let font_size = 12;
            let text_w = measure_text(cstr!(&label), font_size);
            draw_text(
                cstr!(&label),
                (bounds.x + bounds.width / 2.0 - text_w as f32 / 2.0) as i32,
                (bounds.y + bounds.height / 2.0 - 6.0) as i32,
                font_size,
                if is_hovered { YELLOW } else { WHITE },
            );
        }
    }

    // Draw inventory panel using InventoryElement::place!
    let item_names: Vec<String> = view
        .inventory
        .iter()
        .map(|item| item.name.clone())
        .collect();
    let inv_el = InventoryElement::new(item_names);
    inv_el.place(style, "inventory_panel");

    // Draw dialogue box using DialogueElement::place!
    if let Some(dialogue) = &view.dialogue {
        let diag_el = DialogueElement::new(&dialogue.speaker, &dialogue.text);
        diag_el.place(style, "dialogue_box");
    }

    // Render Cursor Tooltip if hovering over hotspot
    if view.dialogue.is_none() {
        if let Some((hotspot, _)) = hovered_hotspot {
            let action_prefix = match hotspot.hover_cursor {
                CursorType::Look => "Look at",
                CursorType::Grab => "Take",
                CursorType::Talk => "Talk to",
                CursorType::ExitLeft | CursorType::ExitRight | CursorType::Walk => "Go to",
                CursorType::Pointer => "Interact with",
            };

            let tooltip_str = format!("{} {}", action_prefix, hotspot.tooltip);
            let font_sz = 14;
            let width = measure_text(cstr!(&tooltip_str), font_sz);

            let box_x = (mouse_pos.x + 15.0) as i32;
            let box_y = (mouse_pos.y + 15.0) as i32;

            let tooltip_bounds = Rectangle {
                x: box_x as f32,
                y: box_y as f32,
                width: (width + 16) as f32,
                height: 24.0,
            };
            unsafe {
                DrawRectangleRec(tooltip_bounds, get_color(0x000000CC));
            }
            draw_rectangle_lines_ex(tooltip_bounds, 1.0, YELLOW);
            draw_text(cstr!(&tooltip_str), box_x + 8, box_y + 5, font_sz, YELLOW);
        }
    }

    end_drawing();
}

fn check_collision_point_rec(point: Vector2, rec: Rectangle) -> bool {
    point.x >= rec.x
        && point.x <= rec.x + rec.width
        && point.y >= rec.y
        && point.y <= rec.y + rec.height
}
