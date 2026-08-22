use muxui::*;

struct Ui {
    attack_buttons: Vec<ButtonElement<UpdateElement<Texture, TextureElement>>>,
    player_hp_bar: UpdateElement<(u32, u32), HpBarElemet>,
    enemy_hp_bar: UpdateElement<(u32, u32), HpBarElemet>,
}

struct Context {
    ui: Ui,
    player_hp: (u32, u32),
    enemy_hp: (u32, u32),
}

fn create_btn_texture() -> Texture {
    unsafe {
        let image = GenImageChecked(140, 40, 32, 32, RED, BLUE);
        let texture = LoadTextureFromImage(image);
        UnloadImage(image);
        texture
    }
}

fn main() {
    App::init(800, 600, "Layout Maker", init_context)
        .on_update(update)
        .set_fps(24)
        .set_style_file("data/layout.ui")
        .run();
}

fn init_context() -> Context {
    Context {
        ui: Ui {
            attack_buttons: vec![
                ButtonElement::new(UpdateElement::new(
                    TextureElement::from(create_btn_texture()),
                    update_texture,
                )),
                ButtonElement::new(UpdateElement::new(
                    TextureElement::from(create_btn_texture()),
                    update_texture,
                )),
                ButtonElement::new(UpdateElement::new(
                    TextureElement::from(create_btn_texture()),
                    update_texture,
                )),
                ButtonElement::new(UpdateElement::new(
                    TextureElement::from(create_btn_texture()),
                    update_texture,
                )),
            ],
            player_hp_bar: UpdateElement::new(
                HpBarElemet {
                    invert: false,
                    current: 0,
                    max: 0,
                },
                update_hp_bar,
            ),
            enemy_hp_bar: UpdateElement::new(
                HpBarElemet {
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
        .place(style, "atk_bar");
        ctx.ui.player_hp_bar.place(style, "player_hp_bar");
        ctx.ui.enemy_hp_bar.place(style, "enemy_hp_bar");
    }
    end_drawing();
}

struct HpBarElemet {
    invert: bool,
    current: u32,
    max: u32,
}

impl Element for HpBarElemet {
    fn draw(&self, position: Vector2, style: &Style, name: &str) {
        unsafe {
            DrawRectangleV(
                position,
                self.measure(style, name),
                get_color_field(style, &[name, "background_color"], WHITE),
            );

            let percent = self.current as f32 / self.max as f32;
            let size = self.measure(style, name);

            let position = if self.invert {
                let Vector2 { x, y } = position;
                let w = size.x;

                let empty_space = w - (w * percent);
                let x = x + empty_space;
                Vector2 { x, y }
            } else {
                position
            };

            let size = {
                let Vector2 { x: w, y: h } = size;

                let w = w * percent;
                Vector2 { x: w, y: h }
            };

            DrawRectangleV(
                position,
                size,
                get_color_field(style, &[name, "foreground_color"], GREEN),
            );
            if let Some(this) = style.get(&[name]) {
                TextElement::new(format!("{}/{}", self.max, self.current)).place(this, "label");
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

fn update_hp_bar(e: &mut HpBarElemet, (current, max): (u32, u32)) {
    e.current = current;
    e.max = max;
}

fn update_texture(e: &mut TextureElement, t: Texture) {
    *e = TextureElement::from(t);
}
