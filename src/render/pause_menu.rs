use raylib::prelude::*;

const MARGIN_X: i32 = 36;
const MARGIN_Y: i32 = 12;

const FONT_SIZE: i32 = 16;

const BUTTON_HEIGHT: i32 = 48;

const BUTTON_FG: Color = Color::WHITE;

const BUTTON_BG_INACTIVE: Color = Color::new(0, 0, 0, 240);
const BUTTON_BG_HOVER: Color = Color::new(64, 128, 192, 240);

const BUTTON_BORDER_COLOR: Color = Color::WHITE;

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

    fn act(&self, rl: &mut RaylibHandle) -> Option<PauseMenuState> {
        match self {
            ButtonType::BTG => Some(PauseMenuState::Running),
            ButtonType::QUIT => Some(PauseMenuState::ShouldQuit),
            ButtonType::VIDEO(video_button_type) => match video_button_type {
                None => Some(PauseMenuState::Video),
                Some(vbt) => vbt.act(rl),
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum VideoButtonType {
    BACK,
    VSYNC,
    FULLSCREEN,
}

impl VideoButtonType {
    fn get_text(&self) -> &str {
        match self {
            VideoButtonType::BACK => "Back",
            VideoButtonType::VSYNC => "Toggle Vertical Sync",
            VideoButtonType::FULLSCREEN => "Toggle Fullscreen",
        }
    }

    fn act(&self, rl: &mut RaylibHandle) -> Option<PauseMenuState> {
        match self {
            VideoButtonType::BACK => Some(PauseMenuState::Paused),
            VideoButtonType::VSYNC => {
                let ws = rl.get_window_state();
                ws.set_vsync_hint(!ws.vsync_hint());
                rl.set_window_state(ws);

                None
            }
            VideoButtonType::FULLSCREEN => {
                rl.toggle_borderless_windowed();

                None
            }
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

    hover: bool,
}

impl Button {
    pub fn new(button: ButtonType) -> Self {
        Button {
            x: -1,
            y: -1,
            width: -1,
            height: BUTTON_HEIGHT,
            button,
            hover: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PauseMenuState {
    Running,
    Paused,
    Video,
    ShouldQuit,
}

trait GuiElement {
    // FIXME: we never use this method (width is calculated when needed, e.g. RowLayout::render)
    fn get_total_width(&self) -> i32;
    fn get_total_height(&self) -> i32;
    fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32);
    fn check_mouse(
        &mut self,
        rl: &mut RaylibHandle,
        mx: i32,
        my: i32,
        lmb_pressed: bool,
    ) -> Option<PauseMenuState>;
}

impl GuiElement for Button {
    fn get_total_width(&self) -> i32 {
        todo!()
    }

    fn get_total_height(&self) -> i32 {
        // FIXME: maybe remove if we only use the constant BUTTON_HEIGHT?
        // although it's likely that this will be useful in the future
        // so actually probably DON't remove this property
        self.height
    }

    fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32) {
        // store current rendering params for mouse inputs
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = BUTTON_HEIGHT;

        d.draw_rectangle(
            x,
            y,
            width,
            BUTTON_HEIGHT,
            if self.hover {
                BUTTON_BG_HOVER
            } else {
                BUTTON_BG_INACTIVE
            },
        );
        d.draw_rectangle_lines(x, y, width, BUTTON_HEIGHT, BUTTON_BORDER_COLOR);

        let text_width = d.measure_text(self.button.get_text(), FONT_SIZE);
        let text_x = (width - text_width) / 2 + x;
        let text_y = (BUTTON_HEIGHT - FONT_SIZE) / 2 + y;
        d.draw_text(self.button.get_text(), text_x, text_y, FONT_SIZE, BUTTON_FG);
    }

    fn check_mouse(
        &mut self,
        rl: &mut RaylibHandle,
        mx: i32,
        my: i32,
        lmb_pressed: bool,
    ) -> Option<PauseMenuState> {
        self.hover = (self.x..(self.x + self.width)).contains(&(mx as i32))
            && (self.y..(self.y + self.height)).contains(&(my as i32));
        if self.hover && lmb_pressed {
            self.button.act(rl)
        } else {
            None
        }
    }
}

struct ColLayout {
    elements: Box<[Box<dyn GuiElement>]>,
}

impl GuiElement for ColLayout {
    fn get_total_width(&self) -> i32 {
        todo!()
    }

    fn get_total_height(&self) -> i32 {
        self.elements
            .iter()
            .map(|e| e.get_total_height())
            .sum::<i32>()
            + (self.elements.len() as i32 - 1) * MARGIN_Y
    }

    fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32) {
        let mut current_y = y;

        // TODO: remove enumerate in refactor
        for (i, element) in self.elements.iter_mut().enumerate() {
            element.render(d, x, current_y, width);

            // bump current y position
            let element_height = element.get_total_height();
            current_y += element_height + MARGIN_Y;
        }
    }

    fn check_mouse(
        &mut self,
        rl: &mut RaylibHandle,
        mx: i32,
        my: i32,
        lmb_pressed: bool,
    ) -> Option<PauseMenuState> {
        // we don't need to do any mouse checks in this struct, but we need to propagate
        self.elements
            .iter_mut()
            .map(|e| e.check_mouse(rl, mx, my, lmb_pressed))
            .filter(|o| o.is_some())
            .last()
            .flatten()
    }
}

struct RowLayout {
    elements: Box<[Box<dyn GuiElement>]>,
}

impl GuiElement for RowLayout {
    fn get_total_width(&self) -> i32 {
        todo!()
    }

    fn get_total_height(&self) -> i32 {
        self.elements
            .iter()
            .map(|e| e.get_total_height())
            .max()
            .unwrap_or(0)
    }

    fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32) {
        // XXX: we calculate element width here
        let num_elements = self.elements.len() as i32;
        let element_width = (width - MARGIN_X * (num_elements - 1)) / num_elements;

        for (i, element) in self.elements.iter_mut().enumerate() {
            let element_x = x + i as i32 * (element_width + MARGIN_X);

            element.render(d, element_x, y, element_width);
        }
    }

    fn check_mouse(
        &mut self,
        rl: &mut RaylibHandle,
        mx: i32,
        my: i32,
        lmb_pressed: bool,
    ) -> Option<PauseMenuState> {
        // we don't need to do any mouse checks in this struct, but we need to propagate
        self.elements
            .iter_mut()
            .map(|e| e.check_mouse(rl, mx, my, lmb_pressed))
            .filter(|o| o.is_some())
            .last()
            .flatten()
    }
}

pub struct PauseMenu {
    state: PauseMenuState,
    root_element: Option<Box<dyn GuiElement>>,
}

impl PauseMenu {
    pub fn new() -> Self {
        // pause by default
        // FIXME: see if there is some way of not having another copy of root_element here
        // and defer everything about root_element to set_state()
        PauseMenu {
            state: PauseMenuState::Paused,
            root_element: Some(Box::new(ColLayout {
                elements: Box::new([
                    Box::new(Button::new(ButtonType::BTG)),
                    Box::new(Button::new(ButtonType::VIDEO(None))),
                    Box::new(Button::new(ButtonType::QUIT)),
                ]),
            })),
        }

        // Set root_element; removes another place to copy root_element by calling set_state
        // FIXME: nvm requires rl which I don't want to introduce to ::new
        // probably some way of fixing this
        // pm.set_state(rl, state);
    }

    fn set_state(&mut self, rl: &mut RaylibHandle, state: PauseMenuState) {
        self.state = state;
        match state {
            PauseMenuState::Running => {
                // FIXME: unnecessary?
                self.root_element = None;

                rl.disable_cursor();
            }
            PauseMenuState::Paused => {
                rl.enable_cursor();

                self.root_element = Some(Box::new(ColLayout {
                    elements: Box::new([
                        Box::new(Button::new(ButtonType::BTG)),
                        Box::new(Button::new(ButtonType::VIDEO(None))),
                        Box::new(Button::new(ButtonType::QUIT)),
                    ]),
                }));
            }
            PauseMenuState::Video => {
                self.root_element = Some(Box::new(ColLayout {
                    elements: Box::new([
                        Box::new(RowLayout {
                            elements: Box::new([
                                Box::new(ColLayout {
                                    elements: Box::new([Box::new(Button::new(ButtonType::VIDEO(
                                        Some(VideoButtonType::VSYNC),
                                    )))]),
                                }),
                                Box::new(ColLayout {
                                    elements: Box::new([Box::new(Button::new(ButtonType::VIDEO(
                                        Some(VideoButtonType::FULLSCREEN),
                                    )))]),
                                }),
                            ]),
                        }),
                        Box::new(Button::new(ButtonType::VIDEO(Some(VideoButtonType::BACK)))),
                    ]),
                }));
            }
            PauseMenuState::ShouldQuit => {}
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) {
        if matches!(rl.get_key_pressed(), Some(KeyboardKey::KEY_ESCAPE)) {
            match self.state {
                PauseMenuState::Running => {
                    self.set_state(rl, PauseMenuState::Paused);
                }
                PauseMenuState::Paused => {
                    self.set_state(rl, PauseMenuState::Running);
                }
                PauseMenuState::Video => {
                    self.set_state(rl, PauseMenuState::Paused);
                }
                PauseMenuState::ShouldQuit => {}
            }
            eprintln!("pause menu: pause toggled");
        }

        let Some(ref mut root_element) = self.root_element else {
            return;
        };

        let Vector2 { x: mx, y: my } = rl.get_mouse_position();

        let lmb_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        if let Some(new_state) = root_element.check_mouse(rl, mx as i32, my as i32, lmb_pressed) {
            self.set_state(rl, new_state);
        }
    }

    pub fn render(&mut self, d: &mut RaylibDrawHandle) {
        if self.state == PauseMenuState::Running {
            return;
        }

        let Some(ref mut root_element) = self.root_element else {
            return;
        };

        let screen_width = d.get_screen_width();
        let screen_height = d.get_screen_height();

        // Tint screen
        d.draw_rectangle(0, 0, screen_width, screen_height, Color::new(0, 0, 0, 127));

        let total_width = screen_width / 2;
        let start_x = (screen_width - total_width) / 2;
        let start_y = (screen_height - root_element.get_total_height()) / 2;

        root_element.render(d, start_x, start_y, total_width);
    }

    pub fn is_running(&self) -> bool {
        self.state == PauseMenuState::Running
    }

    pub fn should_quit(&self) -> bool {
        self.state == PauseMenuState::ShouldQuit
    }
}
