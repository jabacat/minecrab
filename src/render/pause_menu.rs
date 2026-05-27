use raylib::prelude::*;

// useless boilerplate that we could probably do without tbh
#[derive(Clone, Copy, PartialEq)]
enum ButtonType {
    BTG,
    QUIT,
    VIDEO(Option<VideoButtonType>),
}

impl ButtonType {
    fn get_text(&self) -> &str {
        match self {
            ButtonType::BTG => "Back to Game",
            ButtonType::QUIT => "Quit",
            ButtonType::VIDEO(video_button_type) => match video_button_type {
                None => "Video Settings",
                Some(vbt) => vbt.get_text(),
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum VideoButtonType {
    VSYNC,
    FULLSCREEN,
}

impl VideoButtonType {
    fn get_text(&self) -> &str {
        match self {
            VideoButtonType::VSYNC => "Toggle Vertical Sync",
            VideoButtonType::FULLSCREEN => "Toggle Fullscreen",
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

#[derive(PartialEq)]
enum PauseMenuState {
    RUNNING,
    PAUSED,
    VIDEO,
}

pub struct PauseMenu {
    state: PauseMenuState,
    hover: Option<ButtonType>,
    buttons: Vec<Button>,
}

impl PauseMenu {
    pub fn new() -> Self {
        // pause by default
        PauseMenu {
            state: PauseMenuState::PAUSED,
            hover: None,
            buttons: Vec::new(),
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) -> bool {
        if matches!(rl.get_key_pressed(), Some(KeyboardKey::KEY_ESCAPE)) {
            match self.state {
                PauseMenuState::RUNNING => {
                    rl.enable_cursor();
                    self.state = PauseMenuState::PAUSED;
                }
                PauseMenuState::PAUSED => {
                    // FIXME: unnecessary?
                    self.buttons.clear();

                    rl.disable_cursor();
                    self.state = PauseMenuState::RUNNING;
                }
                PauseMenuState::VIDEO => {
                    self.state = PauseMenuState::PAUSED;
                }
            }
            eprintln!("pause menu: pause toggled");
        }

        let Vector2 { x: mx, y: my } = rl.get_mouse_position();

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            // Process button processes
            for b in self.buttons.iter() {
                if (b.x..(b.x + b.width)).contains(&(mx as i32))
                    && (b.y..(b.y + b.height)).contains(&(my as i32))
                {
                    match b.button {
                        ButtonType::BTG => {
                            self.state = PauseMenuState::RUNNING;
                            // FIXME: unnecessary?
                            self.buttons.clear();
                            rl.disable_cursor();
                            return false;
                        }
                        ButtonType::QUIT => {
                            return true;
                        }
                        ButtonType::VIDEO(video_button_type) => match video_button_type {
                            None => {
                                self.state = PauseMenuState::VIDEO;
                            }
                            Some(VideoButtonType::VSYNC) => {
                                let ws = rl.get_window_state();
                                ws.set_vsync_hint(!ws.vsync_hint());
                                rl.set_window_state(ws);
                            }
                            Some(VideoButtonType::FULLSCREEN) => {
                                rl.toggle_borderless_windowed();
                            }
                        },
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
        if self.state == PauseMenuState::RUNNING {
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

        let font_size = 16;

        // this type annotation is pain in my assholes
        let pause_buttons: &[&[ButtonType]] =
            &[&[ButtonType::BTG, ButtonType::VIDEO(None), ButtonType::QUIT]];
        let video_buttons: &[&[ButtonType]] = &[
            &[ButtonType::VIDEO(Some(VideoButtonType::VSYNC))],
            &[ButtonType::VIDEO(Some(VideoButtonType::FULLSCREEN))],
        ];

        let buttons = match self.state {
            PauseMenuState::PAUSED => pause_buttons,
            PauseMenuState::VIDEO => video_buttons,
            _ => &[],
        };

        let total_button_width = screen_width / 2;
        let button_height = 48;

        let button_margin_x = 36;
        let button_margin_y = 12;

        let button_width = (total_button_width - (buttons.len() as i32 - 1) * button_margin_x)
            / buttons.len() as i32;
        let button_y_start = (screen_height - button_height) / 2
            - (buttons.iter().max_by_key(|c| c.len()).unwrap().len() as i32 - 1) / 2
                * (button_height + button_margin_y);
        let button_x_start = (screen_width - total_button_width) / 2;

        for (x, col) in buttons.iter().enumerate() {
            let bx = button_x_start + x as i32 * (button_width + button_margin_x);
            for (y, b) in col.iter().enumerate() {
                let by = button_y_start + y as i32 * (button_height + button_margin_y);
                self.buttons.push(Button {
                    x: bx,
                    y: by,
                    width: button_width,
                    height: button_height,
                    button: *b,
                });

                d.draw_rectangle(
                    bx,
                    by,
                    button_width,
                    button_height,
                    if self.hover == Some(*b) {
                        button_color_active
                    } else {
                        button_color_inactive
                    },
                );
                d.draw_rectangle_lines(bx, by, button_width, button_height, Color::WHITE);

                let text_width = d.measure_text(b.get_text(), font_size);
                let text_x = (button_width - text_width) / 2 + bx;
                let text_y = (button_height - font_size) / 2 + by;
                d.draw_text(b.get_text(), text_x, text_y, font_size, Color::WHITE);
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.state == PauseMenuState::RUNNING
    }
}
