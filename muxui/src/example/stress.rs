use std::time::Instant;
use muxui::*;

struct Context {
    buttons: Vec<ButtonElement<TextElement>>,
    clicks: usize,
}

fn main() {
    App::init(1024, 768, "MuxUI Stress Test", || {
        let mut buttons = Vec::new();
        for i in 0..1000 {
            buttons.push(ButtonElement::new(TextElement::new(format!("{}", i))));
        }
        Context {
            buttons,
            clicks: 0,
        }
    })
    .set_style_file("data/example/stress.gss")
    .on_update(update)
    .run();
}

fn update(ctx: &mut Context, style: &Style) {
    let start = Instant::now();

    // Check clicks on all 1000 buttons
    for (i, btn) in ctx.buttons.iter().enumerate() {
        if btn.click() {
            ctx.clicks += 1;
            println!("Clicked button {}, total clicks: {}", i, ctx.clicks);
        }
    }

    begin_drawing();
    clear_background(BLACK);

    // Build the grid using nested StackLayouts: 25 rows by 40 columns
    let mut row_layouts = Vec::new();
    for r in 0..25 {
        let mut row_children = Vec::new();
        for c in 0..40 {
            let idx = r * 40 + c;
            row_children.push(("btn_style".to_string(), &ctx.buttons[idx] as &dyn Element));
        }
        row_layouts.push(StackLayout::new(row_children));
    }

    let mut main_children = Vec::new();
    for row in &row_layouts {
        main_children.push(("row_stack".to_string(), row as &dyn Element));
    }

    let main_stack = StackLayout::new(main_children);
    main_stack.place(style, "main_stack");

    // Display performance metrics
    let elapsed = start.elapsed();
    let fps = get_fps();
    let stats = format!(
        "FPS: {} | Frame Time: {:.2?} | Clicks: {}",
        fps, elapsed, ctx.clicks
    );
    // draw_text(cstr!(&stats), 10, 740, 20, GREEN);
    TextElement::new(stats).place(style, "stats");

    end_drawing();
}
