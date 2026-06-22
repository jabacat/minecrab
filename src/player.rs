use raylib::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{world::{collision::voxel_raycast, generation::World}};

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
const BLOCK_FRICTION: f32 = 0.25;
const MOUSE_SENS: f32 = 0.005;

// We treat the camera as being near but not at the top of the player.
const PLAYER_HEIGHT: f32 = 1.8;
const CAMERA_HEIGHT: f32 = 1.6;
// Assume a spherical cow ...
const PLAYER_RADIUS: f32 = 0.5;

// From where the player is, get the camera position
pub fn camera_pos_from_player_pos(ppos: Vector3) -> Vector3 {
    ppos + Vector3{x: 0.0, y: CAMERA_HEIGHT, z: 0.0}
}

fn get_input_axis(rl: &mut RaylibHandle, neg: KeyboardKey, pos: KeyboardKey) -> f32 {
    f32::from(rl.is_key_down(pos)) - f32::from(rl.is_key_down(neg))
}

fn movement_smooth(from: f32, to: f32) -> f32 {
    from + (to - from) * FRICTION
}

// TODO: this should probably be named camera
pub struct Player {
    pub camera: Camera3D,
}

impl Player {
    pub fn new(pd: &PlayerData) -> Player {
        let camera =
            Camera3D::perspective(
                camera_pos_from_player_pos(pd.pos),
                camera_pos_from_player_pos(pd.fwd), Vector3::new(0.0, 1.0, 0.0), 45.0);

        Player { camera }
    }

    pub fn reset_player(&mut self, pd: &PlayerData) {
        self.camera.position = camera_pos_from_player_pos(pd.pos);
        self.camera.target = camera_pos_from_player_pos(pd.fwd);
    }

    pub fn update_camera(&mut self, pd: &mut PlayerData, interp: f32, world: &World) {
        let target_pos = pd.prev_pos + (pd.next_pos - pd.prev_pos) * interp;
        pd.pos = target_pos; // Self::next_pos_until_coll(pd.pos, target_pos, world);
        self.camera.position = camera_pos_from_player_pos(pd.pos);

        pd.fwd =
            pd.pos + pd.prev_fwd + (pd.next_fwd - pd.prev_fwd) * interp;
        self.camera.target = camera_pos_from_player_pos(pd.fwd);
    }

    pub fn process_tick(&mut self, pd: &mut PlayerData, rl: &mut RaylibHandle, world: &World) {
        (pd.prev_pos, pd.prev_fwd) = (pd.next_pos, pd.next_fwd);
        self.handle_input(pd, rl, world);
    }

    // This gets the next position, but only steps forward in the requested
    // direction until the player collides with a block. This way we can handle
    // motion appropriately.
    fn next_pos_until_coll(current_pos: Vector3, target_pos: Vector3, world: &World) -> Vector3 {
        // These are all of the "corners" of a player that could collide with something.
        let player_corners = [
            current_pos + Vector3{x: -PLAYER_RADIUS, y: 0.0, z: -PLAYER_RADIUS},
            current_pos + Vector3{x:  PLAYER_RADIUS, y: 0.0, z: -PLAYER_RADIUS},
            current_pos + Vector3{x: -PLAYER_RADIUS, y: 0.0, z:  PLAYER_RADIUS},
            current_pos + Vector3{x:  PLAYER_RADIUS, y: 0.0, z:  PLAYER_RADIUS},
            current_pos + Vector3{x: -PLAYER_RADIUS, y: PLAYER_HEIGHT, z: -PLAYER_RADIUS},
            current_pos + Vector3{x:  PLAYER_RADIUS, y: PLAYER_HEIGHT, z: -PLAYER_RADIUS},
            current_pos + Vector3{x: -PLAYER_RADIUS, y: PLAYER_HEIGHT, z:  PLAYER_RADIUS},
            current_pos + Vector3{x:  PLAYER_RADIUS, y: PLAYER_HEIGHT, z:  PLAYER_RADIUS},
        ];
        let direction = target_pos - current_pos;
        // This is the max possible we could travel.
        let mut distance = direction.length();
        println!("Old distance: {distance}");
        for player_corner in player_corners {
            if let Some(hit) = voxel_raycast(
                world,
               player_corner.x, player_corner.y, player_corner.z,
                direction.x, direction.y, direction.z, Some(distance)
            ) {
                let hit_distance = (hit.raw_coords - player_corner).length();
                if hit_distance < distance {
                    distance = hit_distance;
                }
            }
            
        }
        current_pos + direction * distance
    }

    fn handle_input(&mut self, pd: &mut PlayerData, rl: &mut RaylibHandle, world: &World) {
        let mouse_delta = rl.get_mouse_delta();

        pd.view_azim += mouse_delta.x * MOUSE_SENS;
        pd.view_elev -= mouse_delta.y * MOUSE_SENS;

        // Avoid vertical singularities
        pd.view_elev = pd.view_elev.clamp(-1.57, 1.57);

        if rl.is_key_pressed(keys::SPEED_INC) {
            pd.speed *= 2.0;
        } else if rl.is_key_pressed(keys::SPEED_DEC) {
            pd.speed /= 2.0;
        }

        let (azim_cos, azim_sin) = (pd.view_azim.cos(), pd.view_azim.sin());

        let flat_forward = Vector3 {
            x: azim_cos,
            y: 0.0,
            z: azim_sin,
        };
        let right = Vector3 {
            x: -azim_sin,
            y: 0.0,
            z: azim_cos,
        };

        let (elev_cos, elev_sin) = (pd.view_elev.cos(), pd.view_elev.sin());

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

        let raw_momentum = right * ipx + Vector3::new(0.0, 1.0, 0.0) * ipy + flat_forward * ipz;

        pd.momentum = Vector3 {
            x: movement_smooth(pd.momentum.x, raw_momentum.x),
            y: movement_smooth(pd.momentum.y, raw_momentum.y),
            z: movement_smooth(pd.momentum.z, raw_momentum.z),
        };

        pd.next_pos = Self::next_pos_until_coll(pd.next_pos, pd.next_pos + pd.momentum * pd.speed, world);
        pd.next_fwd = forward;
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlayerData {
    pub pos: Vector3,
    pub fwd: Vector3,
    
    pub prev_pos: Vector3,
    pub next_pos: Vector3,
    pub prev_fwd: Vector3,
    pub next_fwd: Vector3,

    pub speed: f32,
    pub momentum: Vector3,
    pub view_azim: f32,
    pub view_elev: f32,
}

impl PlayerData {
    pub fn new() -> Self {
        let pos = Vector3::new(3.0, 80., 3.0);
        let view_azim: f32 = -2.3;
        let view_elev: f32 = -0.8;

        let target = pos
            + Vector3 {
                x: view_azim.cos() * view_elev.cos(),
                y: view_elev.sin(),
                z: view_azim.sin() * view_elev.cos(),
            };

        PlayerData {
            pos,
            fwd: target,
            prev_pos: pos,
            next_pos: pos,
            prev_fwd: target,
            next_fwd: target,
            speed: DEFAULT_SPEED,
            momentum: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            view_azim,
            view_elev,
        }
    }
}
