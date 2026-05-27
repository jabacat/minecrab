use raylib::prelude::*;

// useless boilerplate that we could probably do without tbh
#[derive(Clone, Copy, PartialEq)]
enum ButtonType {
    BTG,
    QUIT,
}

impl ButtonType {
    fn get_text(&self) -> &str {
        match self {
            ButtonType::BTG => "Back to Game",
            ButtonType::QUIT => "Quit",
        }
    }
}

#[derive(Clone, Copy)]
struct Button {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    button: ButtonType,
}

pub struct PauseMenu {
    pub paused: bool,
    hover: Option<ButtonType>,
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

    pub fn update(&mut self, rl: &mut RaylibHandle) -> bool {
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
                        ButtonType::BTG => {
                            self.paused = false;
                            // FIXME: unnecessary?
                            self.buttons.clear();
                            rl.disable_cursor();
                            return false;
                        }
                        ButtonType::QUIT => {
                            return true;
                        }
                        _ => (),
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

        false
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

        let button_margin_y = 12;

        let font_size = 16;

        let button_width = 640;
        let button_height = 48;
        let button_x = (screen_width - button_width) / 2;

        let buttons = [ButtonType::BTG, ButtonType::QUIT];

        let button_y_start = (screen_height - button_height) / 2
            - (buttons.len() as i32 - 1) / 2 * (button_height + button_margin_y);

        for (i, b) in buttons.iter().enumerate() {
            let by = button_y_start + i as i32 * (button_height + button_margin_y);
            self.buttons.push(Button {
                x: button_x,
                y: by,
                width: button_width,
                height: button_height,
                button: *b,
            });

            d.draw_rectangle(
                button_x,
                by,
                button_width,
                button_height,
                if self.hover == Some(*b) {
                    button_color_active
                } else {
                    button_color_inactive
                },
            );
            d.draw_rectangle_lines(button_x, by, button_width, button_height, Color::WHITE);

            let text_width = d.measure_text(b.get_text(), font_size);
            let text_x = (button_width - text_width) / 2 + button_x;
            let text_y = (button_height - font_size) / 2 + by;
            d.draw_text(b.get_text(), text_x, text_y, font_size, Color::WHITE);
        }
    }
}
