use muxui::*;
use mux_engine::{EngineController, EngineEvent, CursorType};

struct Context {
    engine: EngineController,
}

fn main() {
    App::init(800, 600, "Mux Point-and-Click Engine", init_context)
        .on_update(update)
        .set_fps(30)
        .run();
}

fn init_context() -> Context {
    let scenes_gss = load_style_fallback("data/scenes.gss");
    let mut engine = EngineController::new(scenes_gss, "scene_hallway".to_string());

    // Register script hooks for logic
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

    Context { engine }
}

/// NOTE: This can be put in [`muxui`]
fn load_style_fallback(path: &str) -> Style {
    match gss::load_gss_from_file(path) {
        Ok(style) => {
            println!("Engine: Loaded GSS scene config from: {}", path);
            style
        }
        Err(err) => {
            println!("Engine Error: Could not load scene config {}, error: {}. Using empty layout.", path, err);
            Style::new()
        }
    }
}

fn update(ctx: &mut Context, _style: &Style) {
    let screen_w = get_screen_width() as f32;
    let screen_h = get_screen_height() as f32;

    // 1. Fetch current scene snapshot from the engine
    let view = ctx.engine.current_view(screen_w, screen_h);

    // 2. Process Input and emit Engine Events
    let mouse_pos = get_mouse_position();
    let mut hovered_hotspot = None;

    if view.dialogue.is_some() {
        // Dialogue blocks scene interaction. Any click or space key skips/closes the dialogue.
        if is_mouse_button_pressed(MOUSE_BUTTON_LEFT) || is_key_pressed(KEY_SPACE) {
            ctx.engine.process_event(EngineEvent::SkipDialogue);
        }
    } else {
        // Find if mouse is hovering over any hotspot
        for hotspot in &view.hotspots {
            if check_collision_point_rec(mouse_pos, hotspot.bounds) {
                hovered_hotspot = Some(hotspot);
                break;
            }
        }

        // Trigger action on left click
        if let Some(hotspot) = hovered_hotspot {
            if is_mouse_button_pressed(MOUSE_BUTTON_LEFT) {
                ctx.engine.process_event(EngineEvent::ClickHotspot {
                    hotspot_id: hotspot.id.clone(),
                });
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
    let header_text = format!("Scene: {}", view.scene_id.replace("scene_", "").to_uppercase());
    draw_text(
        cstr!(&header_text),
        20,
        20,
        24,
        LIGHTGRAY,
    );

    // Draw active hotspots (draw borders for development visualization)
    for hotspot in &view.hotspots {
        let is_hovered = hovered_hotspot.map_or(false, |h| h.id == hotspot.id);

        if is_hovered {
            // Draw filled transparent indicator
            unsafe {
                DrawRectangleRec(hotspot.bounds, get_color(0xFFFFFF33));
            }
        }

        // Draw border
        let color = if is_hovered { YELLOW } else { GRAY };
        draw_rectangle_lines_ex(hotspot.bounds, 2.0, color);

        // Draw hotspot label inside/above the bounds
        let label = format!("[{}]", hotspot.tooltip);
        let font_size = 12;
        let text_w = measure_text(cstr!(&label), font_size);
        draw_text(
            cstr!(&label),
            (hotspot.bounds.x + hotspot.bounds.width / 2.0 - text_w as f32 / 2.0) as i32,
            (hotspot.bounds.y + hotspot.bounds.height / 2.0 - 6.0) as i32,
            font_size,
            if is_hovered { YELLOW } else { WHITE },
        );
    }

    // Render Inventory HUD at the top-right
    let mut inv_x = 550;
    let inv_y = 20;
    draw_text(cstr!("INVENTORY:"), inv_x, inv_y, 16, GRAY);
    if view.inventory.is_empty() {
        draw_text(cstr!("(empty)"), inv_x + 100, inv_y, 16, DARKGRAY);
    } else {
        for item in &view.inventory {
            let item_label = format!("[{}]", item.name);
            draw_text(cstr!(&item_label), inv_x + 100, inv_y, 16, GOLD);
            inv_x += 120;
        }
    }

    // Render dialogue overlay at the bottom if active
    if let Some(dialogue) = &view.dialogue {
        let box_y = 440;
        let box_h = 140;
        let box_w = 760;
        let box_x = 20;

        let bounds = Rectangle {
            x: box_x as f32,
            y: box_y as f32,
            width: box_w as f32,
            height: box_h as f32,
        };

        // Draw dialogue container
        unsafe {
            DrawRectangleRec(bounds, get_color(0x0C0C0CFF));
        }
        draw_rectangle_lines_ex(bounds, 2.0, GOLD);

        // Draw speaker name
        let speaker_tag = format!("{}:", dialogue.speaker);
        draw_text(
            cstr!(&speaker_tag),
            box_x + 20,
            box_y + 20,
            20,
            GOLD,
        );

        // Draw dialogue text (with simple wrapping / spacing)
        draw_text(
            cstr!(&dialogue.text),
            box_x + 20,
            box_y + 55,
            18,
            WHITE,
        );

        // Skip instruction prompt
        draw_text(
            cstr!("(Click or Press Space to continue)"),
            box_x + box_w - 240,
            box_y + box_h - 25,
            12,
            GRAY,
        );
    }

    // Render Cursor Tooltip if hovering over hotspot
    if view.dialogue.is_none() {
        if let Some(hotspot) = hovered_hotspot {
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

            // Draw small background for tooltip
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
    point.x >= rec.x && point.x <= rec.x + rec.width && point.y >= rec.y && point.y <= rec.y + rec.height
}
