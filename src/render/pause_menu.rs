use raylib::prelude::*;

// useless boilerplate that we could probably do without tbh
#[derive(Clone, Copy)]
enum Buttons {
    BTG,
    QUIT,
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
                        Buttons::BTG => {
                            self.paused = false;
                            // FIXME: unnecessary?
                            self.buttons.clear();
                            rl.disable_cursor();
                            return false;
                        }
                        Buttons::QUIT => {
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

        let btg_width = 640;
        let btg_height = 48;
        let btg_x = (screen_width - btg_width) / 2;
        let btg_y = (screen_height - btg_height) / 2 - btg_height / 2 - button_margin_y;

        self.buttons.push(Button {
            x: btg_x,
            y: btg_y,
            width: btg_width,
            height: btg_height,
            button: Buttons::BTG,
        });

        d.draw_rectangle(
            btg_x,
            btg_y,
            btg_width,
            btg_height,
            if matches!(self.hover, Some(Buttons::BTG)) {
                button_color_active
            } else {
                button_color_inactive
            },
        );
        d.draw_rectangle_lines(btg_x, btg_y, btg_width, btg_height, Color::WHITE);

        let btg_text_width = d.measure_text("Back to Game", font_size);
        let btg_text_x = (btg_width - btg_text_width) / 2 + btg_x;
        let btg_text_y = (btg_height - font_size) / 2 + btg_y;
        d.draw_text(
            "Back to Game",
            btg_text_x,
            btg_text_y,
            font_size,
            Color::WHITE,
        );

        let quit_width = 640;
        let quit_height = 48;
        let quit_x = (screen_width - quit_width) / 2;
        let quit_y = (screen_height - quit_height) / 2 + quit_height / 2 + button_margin_y;

        self.buttons.push(Button {
            x: quit_x,
            y: quit_y,
            width: quit_width,
            height: quit_height,
            button: Buttons::QUIT,
        });

        d.draw_rectangle(
            quit_x,
            quit_y,
            quit_width,
            quit_height,
            if matches!(self.hover, Some(Buttons::QUIT)) {
                button_color_active
            } else {
                button_color_inactive
            },
        );
        d.draw_rectangle_lines(quit_x, quit_y, quit_width, quit_height, Color::WHITE);

        let font_size = 16;
        let quit_text_width = d.measure_text("Quit", font_size);
        let quit_text_x = (quit_width - quit_text_width) / 2 + quit_x;
        let quit_text_y = (quit_height - font_size) / 2 + quit_y;
        d.draw_text("Quit", quit_text_x, quit_text_y, font_size, Color::WHITE);
    }
}
