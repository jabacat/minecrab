use raylib::prelude::*;

// XXX: Consider importing *?
use crate::render::gui::Button;
use crate::render::gui::ButtonType;
use crate::render::gui::ColLayout;
use crate::render::gui::GuiElement;
use crate::render::gui::RowLayout;

const PAUSE_BG: Color = Color::new(0, 0, 0, 127);

#[derive(Clone, Copy, PartialEq)]
pub enum PauseButtonType {
    BTG,
    QUIT,
    VIDEO(Option<VideoButtonType>),
}

impl ButtonType for PauseButtonType {
    type T = PauseMenuState;

    fn get_text(&self) -> &str {
        match self {
            PauseButtonType::BTG => "Back to Game",
            PauseButtonType::QUIT => "Quit",
            PauseButtonType::VIDEO(video_button_type) => match video_button_type {
                None => "Video Settings",
                Some(vbt) => vbt.get_text(),
            },
        }
    }

    fn act(&self, rl: &mut RaylibHandle) -> Option<PauseMenuState> {
        match self {
            PauseButtonType::BTG => Some(PauseMenuState::Running),
            PauseButtonType::QUIT => Some(PauseMenuState::ShouldQuit),
            PauseButtonType::VIDEO(video_button_type) => match video_button_type {
                None => Some(PauseMenuState::Video),
                Some(vbt) => vbt.act(rl),
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum VideoButtonType {
    BACK,
    VSYNC,
    FULLSCREEN,
}

impl ButtonType for VideoButtonType {
    type T = PauseMenuState;

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

#[derive(Clone, Copy, PartialEq)]
pub enum PauseMenuState {
    Running,
    Paused,
    Video,
    ShouldQuit,
}

pub struct PauseMenu {
    state: PauseMenuState,
    root_element: Option<Box<dyn GuiElement<PauseMenuState>>>,
}

impl PauseMenu {
    pub fn new() -> Self {
        // pause by default
        // FIXME: see if there is some way of not having another copy of root_element here
        // and defer everything about root_element to set_state()
        PauseMenu {
            state: PauseMenuState::Paused,
            root_element: Some(col!([
                button!(PauseButtonType::BTG),
                button!(PauseButtonType::VIDEO(None)),
                button!(PauseButtonType::QUIT),
            ])),
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

                self.root_element = Some(col!([
                    button!(PauseButtonType::BTG),
                    button!(PauseButtonType::VIDEO(None)),
                    button!(PauseButtonType::QUIT),
                ]));
            }
            PauseMenuState::Video => {
                self.root_element = Some(col!([
                    row!([
                        col!([button!(PauseButtonType::VIDEO(Some(
                            VideoButtonType::VSYNC
                        ),))]),
                        col!([button!(PauseButtonType::VIDEO(Some(
                            VideoButtonType::FULLSCREEN
                        ),))]),
                    ]),
                    button!(PauseButtonType::VIDEO(Some(VideoButtonType::BACK))),
                ]));
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
        d.draw_rectangle(0, 0, screen_width, screen_height, PAUSE_BG);

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
