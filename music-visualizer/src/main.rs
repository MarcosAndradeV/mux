use muxui::*;

fn main() {
    AppBuilder::init(800, 600, "Music Visualizer", Context::new)
        .set_style_file("data/music-visualizer/style.gss")
        .set_audio_device()
        .on_update(update)
        .on_drawing_mode(draw)
        .run();
}

struct Context {
    music: MusicResource,
    is_music_playing: bool
}

impl Context {
    fn new() -> Self {
        let music = MusicResource::load_music_stream("data/music-visualizer/Tower of Dreams.mp3")
            .expect("Cannot load \"data/music-visualizer/Tower of Dreams.mp3\"");
        music.play();
        music.set_volume(0.8);
        Self { music, is_music_playing: true }
    }
}

fn update(ctx: &mut Context) {
    ctx.music.update();
    if is_key_pressed(KEY_SPACE) {
        ctx.is_music_playing = !ctx.is_music_playing;
        if ctx.is_music_playing {
            ctx.music.pause();
        } else {
            ctx.music.resume();
        }
    }
}

fn draw(_ctx: &Context, style: &Style) {
    clear_background(DARKGRAY);
    TextElement::new("Hello, world").place(style, "message");
}
