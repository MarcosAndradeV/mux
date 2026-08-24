use muxapp::App;
use muxapp::muxengine::*;
use muxapp::muxui::*;

struct Context {
    background_texture: String,
    background_texture_element: Option<TextureElement>,
}

fn main() {
    App::init(800, 600, "Mux Point-and-Click Engine", init_context)
        .on_update(update)
        .set_script_hook(script_hook)
        .set_style_file("data/layout.gss")
        .set_initial_scene("hallway")
        .set_fps(30)
        .run();
}

fn init_context() -> Context {
    Context {
        background_texture: "".to_string(),
        background_texture_element: None,
    }
}

fn script_hook(state: &mut GameState, script_name: &str) -> Option<String> {
    match script_name {
        "interact_drawer" => {
            if *state.flags.get("has_brass_key").unwrap_or(&false) {
                Some("action:show_dialogue Drawer 'You unlocked the drawer using the brass key!\nInside you found a shiny gemstone! You Win!'".to_string())
            } else {
                Some("action:show_dialogue Drawer 'The drawer is locked tightly. It seems to require a key.'".to_string())
            }
        }
        _ => None,
    }
}

fn update(ctx: &mut Context, engine: &mut EngineController, style: &Style) {
    let view = engine.current_view(style);

    if ctx.background_texture != view.background_texture {
        ctx.background_texture = view.background_texture.clone();
        ctx.background_texture_element = TextureElement::load_from_file(&view.background_texture);
    }

    let mouse_pos = get_virtual_mouse_position();
    let mut hovered_hotspot = None;
    let mut drawable_hotspot = Vec::new();
    let hotspots_style = view.get_hotspots_style(style);

    if view.dialogue.is_some() {
        // Dialogue blocks scene interaction. Any click or space key skips/closes the dialogue.
        if is_mouse_button_pressed(MOUSE_BUTTON_LEFT) || is_key_pressed(KEY_SPACE) {
            engine.process_event(style, EngineEvent::SkipDialogue);
        }
    } else {
        if let Some(hotspots_style) = hotspots_style {
            for hotspot in &view.hotspots {
                // Find if mouse is hovering over any hotspot using HotspotElement boundary check
                let el = HotspotElement::new_from_style(hotspots_style, &hotspot.id);
                if el.hover() {
                    hovered_hotspot = Some(hotspot);
                }

                // Trigger action on left click
                if el.click() {
                    engine.process_event(
                        style,
                        EngineEvent::ClickHotspot {
                            hotspot_id: hotspot.id.clone(),
                        },
                    );
                }
                drawable_hotspot.push(el);
            }
        }
    }

    clear_background(get_color(0x181818FF));

    // Render background (fallback colors based on active scene)
    if view.scene_id == "hallway" {
        clear_background(get_color(0x28201CFF)); // Warm brown/grey for hallway
        if let Some(texture) = &ctx.background_texture_element {
            texture.place(style, "hallway");
        }
    } else if view.scene_id == "kitchen" {
        clear_background(get_color(0x1C2830FF)); // Slate blue for kitchen
    }

    // Draw active hotspots using HotspotElement::place
    if let Some(hotspots_style) = hotspots_style {
        for el in drawable_hotspot {
            el.place(hotspots_style, &el.id);
        }
    }

    // Draw scene header text
    let header_text = format!("Scene: {}", view.scene_id.to_uppercase());
    draw_text(cstr!(&header_text), 20, 20, 24, LIGHTGRAY);

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
        if let Some(hotspot_view) = hovered_hotspot {
            let action_prefix = match hotspot_view.hover_cursor {
                CursorType::Look => "Look at",
                CursorType::Grab => "Take",
                CursorType::Talk => "Talk to",
                CursorType::ExitLeft | CursorType::ExitRight | CursorType::Walk => "Go to",
                CursorType::Pointer => "Interact with",
            };

            let tooltip_str = format!("{} {}", action_prefix, hotspot_view.tooltip);
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
}
