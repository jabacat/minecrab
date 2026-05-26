use raylib::prelude::*;

mod mesh_tools;
mod player;
mod world;

use player::{Player, update_camera};
use world::generation::World;


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

    let mut material = rl.load_material_default(&thread);
    let maps = material.maps_mut();
    maps[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].texture = texture;

    let mut world = World::new();

    let mut frame: i32 = 0;
    
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

            d.draw_mode3D(player.camera, |mut d2, _camera| {
                for (_, chunk) in &world.chunks {
                    d2.draw_mesh(chunk.mesh.as_ref().unwrap(), material.clone(), Matrix::identity());
                }
            });

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
            world.generate_next_chunk();
        }
        frame += 1;
    }
}
