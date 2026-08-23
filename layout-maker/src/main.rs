use muxui::*;

struct Ui {
    attack_buttons: Vec<ButtonElement<RectangleElement>>,
    player_hp_bar: UpdateElement<(u32, u32), HpBarElement>,
    enemy_hp_bar: UpdateElement<(u32, u32), HpBarElement>,
}

struct Context {
    ui: Ui,
    player_hp: (u32, u32),
    enemy_hp: (u32, u32),
}

fn main() {
    App::init(800, 600, "Layout Maker", init_context)
        .on_update(update)
        .set_fps(24)
        .set_style_file("data/layout.gss")
        .run();
}

fn init_context() -> Context {
    Context {
        ui: Ui {
            attack_buttons: vec![
                ButtonElement::new(RectangleElement),
                ButtonElement::new(RectangleElement),
                ButtonElement::new(RectangleElement),
                ButtonElement::new(RectangleElement),
            ],
            player_hp_bar: UpdateElement::new(
                HpBarElement {
                    invert: false,
                    current: 0,
                    max: 0,
                },
                update_hp_bar,
            ),
            enemy_hp_bar: UpdateElement::new(
                HpBarElement {
                    invert: true,
                    current: 0,
                    max: 0,
                },
                update_hp_bar,
            ),
        },
        player_hp: (70, 100),
        enemy_hp: (30, 100),
    }
}

fn update(ctx: &mut Context, style: &Style) {
    for (btn_idx, btn) in ctx.ui.attack_buttons.iter().enumerate() {
        if btn.event() == Event::ButtonClicked {
            println!("Click in button {btn_idx}");
        }
    }
    ctx.ui.player_hp_bar.update(ctx.player_hp);
    ctx.ui.enemy_hp_bar.update(ctx.enemy_hp);
    begin_drawing();
    clear_background(get_color(0x181818FF));
    {
        StackLayout::new(
            ctx.ui
                .attack_buttons
                .iter()
                .map(|e| ("atk_button", e as &dyn Element))
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .place(style, "atk_button_stack");
        StackLayout::new(&[
            ("player_hp_bar", &ctx.ui.player_hp_bar),
            ("vs_text", &TextElement::new("VS")),
            ("enemy_hp_bar", &ctx.ui.enemy_hp_bar),
        ])
        .place(style, "top_hp_bar_stack");
    }
    end_drawing();
}

// type AnimationElement<E> = UpdateElement<(), E>;

struct HpBarElement {
    invert: bool,
    current: u32,
    max: u32,
}

impl Element for HpBarElement {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        unsafe {
            let size = self.measure(style, name);
            DrawRectangleV(
                position,
                size,
                get_color_field(style, &[name, "background_color"], WHITE),
            );

            let percent = self.current as f32 / self.max as f32;

            let position_percent = if self.invert {
                let Vector2 { x, y } = position;
                let w = size.x;

                let empty_space = w - (w * percent);
                let x = x + empty_space;
                Vector2 { x, y }
            } else {
                position
            };

            let size_percent = {
                let Vector2 { x: w, y: h } = size;

                let w = w * percent;
                Vector2 { x: w, y: h }
            };

            DrawRectangleV(
                position_percent,
                size_percent,
                get_color_field(style, &[name, "foreground_color"], GREEN),
            );
            if let Some(this) = style.get(&[name]) {
                TextElement::new(format!("{}/{}", self.max, self.current)).place(this, "hp_text");
            }
        };
    }

    fn measure(&self, style: &Style, name: &str) -> Vector2 {
        Vector2::new(
            style.get_or_default(&[name, "width"]),
            style.get_or_default(&[name, "height"]),
        )
    }
}

fn update_hp_bar(e: &mut HpBarElement, (current, max): (u32, u32)) {
    e.current = current;
    e.max = max;
}
