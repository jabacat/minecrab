use raylib::prelude::*;

// useless boilerplate that we could probably do without tbh
#[derive(Clone, Copy)]
enum Buttons {
    BTG,
}

#[derive(Clone, Copy)]
struct Button {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    button: Buttons,
}

pub struct PauseMenu {
    pub paused: bool,
    hover: Option<Buttons>,
    buttons: Vec<Button>,
}

impl PauseMenu {
    pub fn new() -> Self {
        // pause by default
        PauseMenu {
            paused: true,
            hover: None,
            buttons: Vec::new(),
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) {
        if matches!(rl.get_key_pressed(), Some(KeyboardKey::KEY_ESCAPE)) {
            self.paused ^= true;

            if self.paused {
                rl.enable_cursor();
            } else {
                // FIXME: unnecessary?
                self.buttons.clear();

                rl.disable_cursor();
            }
            eprintln!("pause menu: pause toggled");
        }

        let Vector2 { x: mx, y: my } = rl.get_mouse_position();

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            // Process button processes
            for b in self.buttons.iter() {
                if (b.x..(b.x + b.width)).contains(&(mx as i32))
                    && (b.y..(b.y + b.height)).contains(&(my as i32))
                {
                    match b.button {
                        Buttons::BTG => {
                            self.paused = false;
                            // FIXME: unnecessary?
                            self.buttons.clear();
                            rl.disable_cursor();
                            return;
                        }
                    }
                }
            }
        }

        self.hover = self
            .buttons
            .iter()
            .filter(|b| {
                (b.x..(b.x + b.width)).contains(&(mx as i32))
                    && (b.y..(b.y + b.height)).contains(&(my as i32))
            })
            .map(|b| b.button)
            .next();
    }

    pub fn render(&mut self, d: &mut RaylibDrawHandle) {
        if !self.paused {
            return;
        }

        let screen_width = d.get_screen_width();
        let screen_height = d.get_screen_height();

        // Tint screen
        d.draw_rectangle(0, 0, screen_width, screen_height, Color::new(0, 0, 0, 127));

        // TODO: either use screen size breakpoints or UI scaling to control button sizes
        self.buttons.clear();

        let button_color_inactive = Color::new(0, 0, 0, 240);
        let button_color_active = Color::new(64, 128, 192, 240);

        let button_width = 640;
        let button_height = 48;
        let button_x = (screen_width - button_width) / 2;
        let button_y = (screen_height - button_height) / 2;

        self.buttons.push(Button {
            x: button_x,
            y: button_y,
            width: button_width,
            height: button_height,
            button: Buttons::BTG,
        });

        d.draw_rectangle(
            button_x,
            button_y,
            button_width,
            button_height,
            if matches!(self.hover, Some(Buttons::BTG)) {
                button_color_active
            } else {
                button_color_inactive
            },
        );
        d.draw_rectangle_lines(
            button_x,
            button_y,
            button_width,
            button_height,
            Color::WHITE,
        );

        let font_size = 16;
        let text_width = d.measure_text("Back to Game", font_size);
        let text_x = (screen_width - text_width) / 2;
        let text_y = (screen_height - font_size) / 2;
        d.draw_text("Back to Game", text_x, text_y, font_size, Color::WHITE);
    }
}
