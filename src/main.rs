use raylib::prelude::*;

mod player;
mod render;
mod world;

use player::{Player, update_camera_angle, update_camera_position};
use world::generation::World;

use crate::render::mesh_tools;
use crate::render::pause_menu::PauseMenu;
use crate::render::worldmesh::WorldRenderer;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

const TICK_LENGTH: f32 = 0.025; // 40 ticks per second

// Generate one chunk every [FRAMES_PER_CHUNK] frames so world generation isn't
// exceedingly laggy at the beginning.
const FRAMES_PER_CHUNK: i32 = 5;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Minecrab")
        .vsync()
        .highdpi()
        .build();

    // Disable exit on esc (default raylib behavior)
    rl.set_exit_key(Some(KeyboardKey::KEY_NULL));

    let mut player = Player::new();

    let mut t = rl
        .load_texture(&thread, "assets/full-textures.png")
        .expect("Should load 'assets/full-textures.png'.");

    t.gen_texture_mipmaps();

    let texture: ffi::Texture = unsafe { t.unwrap() };

    let mut material = rl.load_material_default(&thread);
    let maps = material.maps_mut();
    maps[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].texture = texture;

    let mut world = World::new();
    let mut world_renderer: WorldRenderer = WorldRenderer::new(material);

    let mut frame: i32 = 0;

    let mut window_should_close = false;
    let mut pause_menu = PauseMenu::new();
    let mut debug_display = false; // toggle

    let mut update_camera_in = 0_f32; // time until we run update_camera()

    while !window_should_close {
        window_should_close |= rl.window_should_close();
        window_should_close |= pause_menu.update(&mut rl);

        if pause_menu.is_running() {
            // rl.update_camera(&mut camera, CameraMode::CAMERA_FIRST_PERSON);
            update_camera_in -= rl.get_frame_time();
            update_camera_angle(&mut player, &mut rl);
            while update_camera_in < 0_f32 {
                update_camera_position(&mut player, &mut rl);
                update_camera_in += TICK_LENGTH;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_BACKSLASH) { // toggle debug menu
            debug_display = !debug_display;
        }


        rl.draw(&thread, |mut d| {
            d.clear_background(Color::LIGHTBLUE);

            world_renderer.render(&mut d, player.camera);

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

            // Render pause menu
            pause_menu.render(&mut d);
        });

        if frame % FRAMES_PER_CHUNK == 0 {
            world.generate_next_chunk(&mut world_renderer);
        }
        frame += 1;
    }
}
