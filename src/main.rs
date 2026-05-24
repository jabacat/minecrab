use raylib::prelude::*;

use crate::world::generation::generate_chunk;
use crate::world::world::World;

mod mesh_tools;
mod camera_controls;
mod world;

use camera_controls::{Player, update_camera};


const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

// Generate one chunk every [FRAMES_PER_CHUNK] frames so world generation isn't
// exceedingly laggy at the beginning.
const FRAMES_PER_CHUNK: i32 = 5;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Hello, world!")
        .vsync()
        .highdpi()
        .build();

    let mut player = Player::new();

    let mut first_click = false;
    let mut debug_display = false; // toggle

    let texture: ffi::Texture = unsafe {
        let mut t = rl.load_texture(&thread, "assets/full-textures.png").unwrap();
        t.gen_texture_mipmaps();
        t.unwrap()
    };

    let mut world: World = World::new();
    // FIXME: I will be back one day, borrow checker...
    // let rl_ref = & rl;
    // let thread_ref = &thread;
    // let mut models: Vec<Model> = (-8..8).flat_map(|cx| (-8..8).flat_map(move |cy| (-8..8).map(move |cz| generate_chunk(rl_ref, thread_ref, cx, cy, cz)))).collect();

    let mut world = World::new();

    let mut frame = 0;

    while !rl.window_should_close() {
        // require a click on the window before updating camera so the camera
        // doesn't fly away when the cursor enters the window at first
        if !first_click {
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                first_click = true;
                rl.disable_cursor();
            }
        } else {
            // rl.update_camera(&mut camera, CameraMode::CAMERA_FIRST_PERSON);
            update_camera(&mut player, &mut rl);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_BACKSLASH) && first_click { // toggle debug menu
            debug_display = !debug_display;
        }


        rl.draw(&thread, |mut d| {
            d.clear_background(Color::LIGHTBLUE);

            world.render(&mut d, player.camera);

            if !first_click {
                d.draw_text("WIP: Click to start updating camera", 20, 20, 16, Color::DARKGREEN);
            }
            if debug_display {
                let mut debug_info = String::new();
                debug_info += &format!(
                    "Camera position: {:.4} {:.4} {:.4}\n",
                    player.camera.position.x,
                    player.camera.position.y,
                    player.camera.position.z
                );
                debug_info += &format!(
                    "FPS: {}\n",
                    d.get_fps()
                );
                d.draw_text(&debug_info, 20, 20, 16, Color::DARKGREEN);
            }
        });

        if frame % FRAMES_PER_CHUNK == 0 {
            world.generate_next_chunk(&mut rl, &thread, texture);
        }
        frame += 1;

    }
}
