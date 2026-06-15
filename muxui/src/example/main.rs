use std::time::{Instant, SystemTime};

use muxui::*;

struct Context {
    show_clock: bool,
    timer: Instant,
    penger: ButtonElement<TextureElement>,
    message: TextElement,
}

fn main() {
    App::init(800, 600, "Test", || {
        let penger = TextureElement::load_from_file("data/example/penger.png")
            .expect("Could not load texture from \"data/example/penger.png\"");
        Context {
            show_clock: true,
            timer: Instant::now(),
            penger: ButtonElement::new(penger),
            message: TextElement::new("Hello world"),
        }
    })
    .set_style_file("data/example/style.gss")
    .on_update(update)
    .run();
}

fn update(ctx: &mut Context, style: &Style) {
    if is_key_pressed(KEY_R) {
        ctx.timer = Instant::now();
    }
    if is_key_pressed(KEY_C) {
        ctx.show_clock = !ctx.show_clock;
    }
    if ctx.penger.click(style, "penger") {
        println!("You found a penger!");
    }

    begin_drawing();
    clear_background(DARKGRAY);

    ctx.message.place(style, "message");

    let secs = Instant::now().duration_since(ctx.timer).as_secs();
    TextElement::new(format_time(secs)).place(style, "timer");
    if ctx.show_clock {
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - 3 * 3600;
        TextElement::new(format_time(secs)).place(style, "clock");
    }

    ctx.penger.place(style, "penger");

    if is_cursor_on_screen() {
        draw_circle_v(get_mouse_position(), 2.0, RED);
    }
    end_drawing();
}

fn format_time(secs: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}
