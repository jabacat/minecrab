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

// "Player". Controls the current position and their momentum.
pub struct Player {
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

        return Player {
            camera: Camera3D::perspective(
                pos, target,
                Vector3::new(0.0, 1.0, 0.0),
                45.0,
            ),       
            speed: DEFAULT_SPEED,
            momentum: Vector3{x: 0.0, y: 0.0, z: 0.0},
            view_azim,
            view_elev
        };
    }
}

fn get_input_axis(rl: &mut RaylibHandle, neg: KeyboardKey, pos: KeyboardKey) -> f32 {
    f32::from(rl.is_key_down(pos)) - f32::from(rl.is_key_down(neg))
}

/* step from the current velocity towards the new velocity by FRICTION */
fn movement_smooth(from: f32, to: f32) -> f32 {
    from + (to - from) * FRICTION
}

pub fn update_camera_angle(player: &mut Player, rl: &mut RaylibHandle) {
    let mouse_delta = rl.get_mouse_delta();

    player.view_azim += mouse_delta.x * MOUSE_SENS;
    player.view_elev -= mouse_delta.y * MOUSE_SENS;

    // Avoid vertical singularities
    player.view_elev = player.view_elev.clamp(-1.57, 1.57);

    if rl.is_key_pressed(keys::SPEED_INC) { player.speed *= 2.0; }
    else if rl.is_key_pressed(keys::SPEED_DEC) { player.speed /= 2.0; }
}

pub fn update_camera_position(player: &mut Player, rl: &mut RaylibHandle) {
    let (azim_cos, azim_sin) = (player.view_azim.cos(), player.view_azim.sin());

    let flat_forward = Vector3 { x: azim_cos, y: 0.0, z: azim_sin };
    let right = Vector3 { x: -azim_sin, y: 0.0, z: azim_cos };
    
    let (elev_cos, elev_sin) = (player.view_elev.cos(), player.view_elev.sin());

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

    player.momentum = Vector3 {
        x: movement_smooth(player.momentum.x, raw_momentum.x),
        y: movement_smooth(player.momentum.y, raw_momentum.y),
        z: movement_smooth(player.momentum.z, raw_momentum.z),
    };
    
    player.camera.position += player.momentum * player.speed;
    player.camera.target = player.camera.position + forward;
}
