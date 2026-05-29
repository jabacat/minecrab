use raylib::prelude::*;

// XXX: Consider importing *?
use crate::render::gui::Button;
use crate::render::gui::ColLayout;
use crate::render::gui::GuiElement;
use crate::render::gui::RowLayout;

const PAUSE_BG: Color = Color::new(0, 0, 0, 127);

#[derive(Clone, Copy, PartialEq)]
pub enum PauseButtonType {
    BackToGame,
    Quit,
    Video,
    VideoBack,
    VideoVsync,
    VideoFullscreen,
}

impl PauseButtonType {
    fn get_text(&self) -> &str {
        match self {
            // Main pause menu
            PauseButtonType::BackToGame => "Back to Game",
            PauseButtonType::Quit => "Quit",
            PauseButtonType::Video => "Video Settings",

            // Video sub-menu
            PauseButtonType::VideoBack => "Back",
            PauseButtonType::VideoVsync => "Toggle Vertical Sync",
            PauseButtonType::VideoFullscreen => "Toggle Fullscreen",
        }
    }

    fn get_act(&self) -> Box<dyn Fn(&mut RaylibHandle) -> Option<PauseMenuState>> {
        Box::new(match self {
            PauseButtonType::BackToGame => |_| Some(PauseMenuState::Running),
            PauseButtonType::Quit => |_| Some(PauseMenuState::ShouldQuit),
            PauseButtonType::Video => |_| Some(PauseMenuState::Video),
            PauseButtonType::VideoBack => |_| Some(PauseMenuState::Paused),
            PauseButtonType::VideoVsync => |rl| {
                let ws = rl.get_window_state();
                ws.set_vsync_hint(!ws.vsync_hint());
                rl.set_window_state(ws);

                None
            },
            PauseButtonType::VideoFullscreen => |rl| {
                rl.toggle_borderless_windowed();

                None
            },
        })
    }
}

// Macro from creating a button in the pause menu using PauseButtonType
macro_rules! pb {
    ( $e:expr ) => {
        button!($e.get_text(), $e.get_act())
    };
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
    last_mouse_pos: Option<Vector2>,
}

impl PauseMenu {
    pub fn new() -> Self {
        // pause by default
        // FIXME: see if there is some way of not having another copy of root_element here
        // and defer everything about root_element to set_state()
        PauseMenu {
            state: PauseMenuState::Paused,
            root_element: Some(col!([
                pb!(PauseButtonType::BackToGame),
                pb!(PauseButtonType::Video),
                pb!(PauseButtonType::Quit),
            ])),
            last_mouse_pos: None,
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

                // Save mouse position
                self.last_mouse_pos = Some(rl.get_mouse_position());

                rl.disable_cursor();

                // XXX: For some reason, setting the cursor to (0,0) seems to fix
                // the camera explosion
                rl.set_mouse_position((0., 0.));
            }
            PauseMenuState::Paused => {
                rl.enable_cursor();
                // Restore mouse position
                if let Some(lmp) = self.last_mouse_pos {
                    rl.set_mouse_position(lmp);
                    self.last_mouse_pos = None;
                }

                self.root_element = Some(col!([
                    pb!(PauseButtonType::BackToGame),
                    pb!(PauseButtonType::Video),
                    pb!(PauseButtonType::Quit),
                ]));
            }
            PauseMenuState::Video => {
                self.root_element = Some(col!([
                    row!([
                        col!([pb!(PauseButtonType::VideoVsync)]),
                        col!([pb!(PauseButtonType::VideoFullscreen)]),
                    ]),
                    pb!(PauseButtonType::VideoBack),
                ]));
            }
            PauseMenuState::ShouldQuit => {}
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) {
        if rl.get_key_pressed() == Some(KeyboardKey::KEY_ESCAPE) {
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

        // XXX: Convert mouse coordinates from screen to render space
        // SDL reports event coordinates (i.e. mouse position) in screen
        // coordinates, while rendering things in render coordinates when under
        // high DPI scaling. The latter allows for us to draw 1:1 with the
        // physical pixels on the screen, rather than get scaled by the OS.
        // SDL exposes some functions to convert between the two, but raylib
        // doesn't seem to be using them properly (upstream issue).
        let render_scale = rl.get_render_width() as f32 / rl.get_screen_width() as f32;
        let (mx, my) = (mx * render_scale, my * render_scale);

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

        let screen_width = d.get_render_width();
        let screen_height = d.get_render_height();

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
