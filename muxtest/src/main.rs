use muxapp::{App, muxui::*};

#[derive(Default)]
struct AppState {
    counter: i32,
    bg_color: Color
}

fn main() {
    App::init(800, 600, "Mux UI Showcase", AppState::default)
        .set_style_file("./data/test.gss")
        .set_fps(30)
        .on_load(load)
        .on_update(update)
        .run();
}

fn load(state: &mut AppState, style: &Style) {
    state.bg_color = get_color_field(style, &["background_color"], state.bg_color);
}

fn update(state: &mut AppState, style: &Style) {
    clear_background(state.bg_color);
    // 1. Responsive percentage-positioned headers
    TextElement::new("Mux UI Framework").place(style, "title");
    TextElement::new("Responsive % Layouts - GSS Styling - Stateless Buttons").place(style, "subtitle");

    // 2. Reactive state presentation
    let count_label = format!("Current Count: {}", state.counter);
    TextElement::new(count_label).place(style, "counter_display");

    // 3. Standalone button
    if Button::new("Click Me (+1)").place(style, "main_btn").clicked {
        state.counter += 1;
    }

    // 4. Buttons in a StackLayout - Interaction detected immediately after place()!
    let inc10 = Button::new("+10");
    let dec5 = Button::new("-5");
    let reset = Button::new("Reset");

    StackLayout::new(&[
        ("btn_inc10", &inc10),
        ("btn_dec5", &dec5),
        ("btn_reset", &reset),
    ])
    .place(style, "btn_row");

    if inc10.clicked() {
        state.counter += 10;
    }
    if dec5.clicked() {
        state.counter -= 5;
    }
    if reset.clicked() {
        state.counter = 0;
    }

    // 5. Edge-anchored footer (bottom = 0, width = 100%)
    RectangleElement.place(style, "footer_bar");
    TextElement::new("Press F5 or modify data/test.gss to live hotreload styles").place(style, "footer_text");
}
