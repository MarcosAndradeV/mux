use std::time::{Instant, SystemTime};

use muxui::*;
use raylib::*;

fn main() {
    App::init(800, 600, "Test", Context::new())
        .set_style_file("data/style.gss")
        .on_update(update)
        .on_drawing_mode(draw)
        .run();
}

struct Context {
    show_clock: bool,
    timer: Instant,
}

impl Context {
    fn new() -> Self {
        Self {
            show_clock: true,
            timer: Instant::now(),
        }
    }
}

fn update(ctx: &mut Context) {
    if is_key_pressed(KEY_R) {
        ctx.timer = Instant::now();
    }
    if is_key_pressed(KEY_C) {
        ctx.show_clock = !ctx.show_clock;
    }
}

fn draw(ctx: &Context, style: &Style) {
    clear_background(DARKGRAY);
    place_element(style, "message", TextElement::new("Hello, world"));
    if ctx.show_clock {
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            - 3 * 3600;
        place_element(style, "clock", TimerElement(secs));
    } else {
        place_element(style, "timer", TimerElement(ctx.timer.elapsed().as_secs()));
    }
}

pub struct TimerElement(pub u64);

impl Element for TimerElement {
    fn draw(&self, position: Vector2, factor: f32, color: Color) {
        let secs = self.0;
        let text = format!(
            "{:02}:{:02}:{:02}",
            (secs / 3600) % 24,
            (secs / 60) % 60,
            secs % 60
        );
        TextElement::new(text).draw(position, factor, color);
    }

    fn measure(&self, factor: f32) -> Vector2 {
        let text = format!("00:00:00");
        TextElement::new(text).measure(factor)
    }
}
