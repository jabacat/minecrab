use raylib::prelude::*;

pub struct PauseMenu {
    pub paused: bool
}

impl PauseMenu {
    pub fn new() -> Self {
        // pause by default
        PauseMenu { paused: true }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) {
        if matches!(rl.get_key_pressed(), Some(KeyboardKey::KEY_ESCAPE)) {
            self.paused ^= true;
            eprintln!("pause menu: pause toggled");
        } else {
            return;
        }

        if self.paused {
            rl.enable_cursor();
        } else {
            rl.disable_cursor();
        }
    }

    pub fn render(&self, d: &mut RaylibDrawHandle) {
        if !self.paused {
            return;
        }

        d.draw_rectangle(0, 0, d.get_screen_width(), d.get_screen_width(), Color::new(0, 0, 0, 127));
    }
}
