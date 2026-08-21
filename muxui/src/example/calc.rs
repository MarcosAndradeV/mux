use muxui::*;

fn main() {
    App::init(400, 600, "Calc", || Context {
        buttons: vec![
            vec![
                ButtonElement::new(TextElement::new("7")),
                ButtonElement::new(TextElement::new("8")),
                ButtonElement::new(TextElement::new("9")),
            ],
            vec![
                ButtonElement::new(TextElement::new("4")),
                ButtonElement::new(TextElement::new("5")),
                ButtonElement::new(TextElement::new("6")),
            ],
            vec![
                 ButtonElement::new(TextElement::new("1")),
                 ButtonElement::new(TextElement::new("2")),
                 ButtonElement::new(TextElement::new("3")),
            ],
            vec![
                ButtonElement::new(TextElement::new("0")),
                ButtonElement::new(TextElement::new("+")),
                ButtonElement::new(TextElement::new("=")),
            ],
        ],
    })
    .set_style_file("data/example/calc.gss")
    .on_update(|c, s| {
        let mut row_layouts = Vec::new();
        for r in &c.buttons {
            let mut row_children = Vec::new();
            for v in r {
                if v.click() {
                    println!("click")
                }
                row_children.push(("btn_style".to_string(), v as &dyn Element));
            }
            row_layouts.push(StackLayout::new(row_children));
        }

        let mut main_children = Vec::new();
        for row in &row_layouts {
            main_children.push(("row_stack".to_string(), row as &dyn Element));
        }

        let main_stack = StackLayout::new(main_children);

        begin_drawing();
        clear_background(DARKGRAY);
        main_stack.place(s, "main_stack");

        end_drawing();
    })
    .run();
}

struct Context {
    buttons: Vec<Vec<ButtonElement<TextElement>>>,
}
