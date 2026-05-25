use raylib::prelude::*;

mod mesh_tools;
mod camera_controls;
mod world;

use camera_controls::{Player, update_camera};
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
    eprintln!("[minecrab] creating world...");

    let mut world = World::new();
    let mut models: Vec<Model> = Vec::new();

    eprintln!("[minecrab] generating terrain...");
    let r = 0..4;
    for cx in r.clone() {
    for cy in r.clone() {
    for cz in r.clone() {
        world.generate_terrain_chunk(cx, cy, cz);
    }}}

    eprintln!("[minecrab] building meshes...");
    for cx in r.clone() {
    for cy in r.clone() {
    for cz in r.clone() {
        let mesh = world.build_geometry_chunk(cx, cy, cz);
        let model =
            rl.load_model_from_mesh(&thread, unsafe { mesh.make_weak() })
            .unwrap();
        models.push(model);
    }}}

    models.iter_mut().for_each(|model| {
        let materials = model.materials_mut();
        let material = &mut materials[0];
        let maps = material.maps_mut();
        maps[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].texture = texture;
    });
    
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
                for model in &models {
                    d2.draw_model(model, Vector3::zero(), 1., Color::WHITE);
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

        /*
        if frame % FRAMES_PER_CHUNK == 0 {
            world.generate_next_chunk(&mut rl, &thread, texture);
        }
        frame += 1;
        */

    }
}
