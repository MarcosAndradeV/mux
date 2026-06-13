use muxui::*;

fn main() {
    App::init(800, 600, "Music Visualizer", Context::new)
        .set_style_file("data/music-visualizer/style.gss")
        .on_update(update)
        .on_drawing_mode(draw)
        .run();
}

struct Context {}

impl Context {
    fn new() -> Self {
        Self {}
    }
}

fn update(_ctx: &mut Context) {}

fn draw(_ctx: &Context, style: &Style) {
    clear_background(DARKGRAY);
    TextElement::new("Hello, world").place(style, "message");
}
