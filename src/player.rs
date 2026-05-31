use raylib::prelude::*;

mod keys {
    use raylib::prelude::{KeyboardKey, KeyboardKey::*};
    pub const FORW: KeyboardKey = KEY_W;
    pub const BACK: KeyboardKey = KEY_S;
    pub const LEFT: KeyboardKey = KEY_A;
    pub const RIGH: KeyboardKey = KEY_D;
    pub const UPPP: KeyboardKey = KEY_SPACE;
    pub const DOWN: KeyboardKey = KEY_LEFT_SHIFT;

    pub const SPEED_DEC: KeyboardKey = KEY_LEFT_BRACKET;
    pub const SPEED_INC: KeyboardKey = KEY_RIGHT_BRACKET;
}

const DEFAULT_SPEED: f32 = 0.2;
const FRICTION: f32 = 0.15;
const MOUSE_SENS: f32 = 0.005;

fn get_input_axis(rl: &mut RaylibHandle, neg: KeyboardKey, pos: KeyboardKey) -> f32 {
    f32::from(rl.is_key_down(pos)) - f32::from(rl.is_key_down(neg))
}

fn movement_smooth(from: f32, to: f32) -> f32 {
    from + (to - from) * FRICTION
}

pub struct Player {
    pub prev_pos: Vector3,
    pub next_pos: Vector3,
    pub prev_fwd: Vector3,
    pub next_fwd: Vector3,

    pub camera: Camera3D,
    pub speed: f32,
    pub momentum: Vector3,
    pub view_azim: f32,
    pub view_elev: f32,
}

impl Player {
    pub fn new() -> Player {
        let pos = Vector3::new(3.0, 80., 3.0);
        let view_azim: f32 = -2.3;
        let view_elev: f32 = -0.8;

        let target = pos + Vector3 {
                x: view_azim.cos() * view_elev.cos(),
                y: view_elev.sin(),
                z: view_azim.sin() * view_elev.cos()
            };

        let camera = Camera3D::perspective(
                pos, target,
                Vector3::new(0.0, 1.0, 0.0),
                45.0,
            );

        return Player {
            prev_pos: pos,
            next_pos: pos,
            prev_fwd: target,
            next_fwd: target,
            camera,
            speed: DEFAULT_SPEED,
            momentum: Vector3{x: 0.0, y: 0.0, z: 0.0},
            view_azim,
            view_elev
        };
    }

    pub fn update_camera(&mut self, interp: f32) {
        self.camera.position =
            self.prev_pos + (self.next_pos - self.prev_pos) * interp;
        
        self.camera.target =
            self.camera.position
            + self.prev_fwd + (self.next_fwd - self.prev_fwd) * interp;
    }

    pub fn process_tick(&mut self, rl: &mut RaylibHandle) {
        (self.prev_pos, self.prev_fwd) = (self.next_pos, self.next_fwd);
        self.handle_input(rl);
    }

    fn handle_input(&mut self, rl: &mut RaylibHandle) {
        let mouse_delta = rl.get_mouse_delta();

        self.view_azim += mouse_delta.x * MOUSE_SENS;
        self.view_elev -= mouse_delta.y * MOUSE_SENS;

        // Avoid vertical singularities
        self.view_elev = self.view_elev.clamp(-1.57, 1.57);

        if rl.is_key_pressed(keys::SPEED_INC) { self.speed *= 2.0; }
        else if rl.is_key_pressed(keys::SPEED_DEC) { self.speed /= 2.0; }

        let (azim_cos, azim_sin) = (self.view_azim.cos(), self.view_azim.sin());

        let flat_forward = Vector3 { x: azim_cos, y: 0.0, z: azim_sin };
        let right = Vector3 { x: -azim_sin, y: 0.0, z: azim_cos };
        
        let (elev_cos, elev_sin) = (self.view_elev.cos(), self.view_elev.sin());

        let forward = Vector3 {
            x: azim_cos * elev_cos,
            y: elev_sin,
            z: azim_sin * elev_cos,
        };

        let ipx = get_input_axis(rl, keys::LEFT, keys::RIGH);
        let ipy = get_input_axis(rl, keys::DOWN, keys::UPPP);
        let ipz = get_input_axis(rl, keys::BACK, keys::FORW);

        /* for consistent horizontal speed on diagonals. vertical doesn't
         * count because i don't feel like it should */
        let (ipx, ipy) = if ipx.abs() + ipy.abs() > 1.0 {
            (ipx * 0.707, ipy * 0.707)
        } else {
            (ipx, ipy)
        };
        
        let raw_momentum = 
            right * ipx
            + Vector3::new(0.0, 1.0, 0.0) * ipy
            + flat_forward * ipz;

        self.momentum = Vector3 {
            x: movement_smooth(self.momentum.x, raw_momentum.x),
            y: movement_smooth(self.momentum.y, raw_momentum.y),
            z: movement_smooth(self.momentum.z, raw_momentum.z),
        };
        
        self.next_pos += self.momentum * self.speed;
        self.next_fwd = forward;
    }
}
