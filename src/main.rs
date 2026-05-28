use std::fs;

use raylib::prelude::*;

mod player;
mod render;
mod world;

use player::{Player, update_camera_angle, update_camera_position};
use serde::Serialize;
use world::generation::World;
use world::collision::voxel_raycast;

use crate::render::{mesh_tools, worldmesh};
use crate::render::worldmesh::WorldRenderer;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

const TICK_LENGTH: f32 = 0.025; // 40 ticks per second

// Generate one chunk every [FRAMES_PER_CHUNK] frames so world generation isn't
// exceedingly laggy at the beginning.
const FRAMES_PER_CHUNK: i32 = 5;

fn tick(
    world: &mut World, player: &mut Player, rl: &mut RaylibHandle
) {
    update_camera_position(player, rl);
    //terrain generation should be in here too, and a lot of other stuff.
    //probably need some kind of (dreaded) GameState object to keep the
    //parameter list from being ridiculous.
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Minecrab")
        .vsync()
        .highdpi()
        .build();

    let mut player = Player::new();

    let mut first_click = false;
    let mut debug_display = false; // toggle

    let mut next_tick_in = 0_f32; // time until we run update_camera()

    let audio_stream = RaylibAudio::init_audio_device().expect("Can init audio.");
    let open_sound = audio_stream.new_sound(&"assets/audio/menu-open.ogg").expect("Load sound");
    let close_sound = audio_stream.new_sound(&"assets/audio/menu-close.ogg").expect("Load sound");

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
    
    while !rl.window_should_close() {
        // require a click on the window before updating camera so the camera
        // doesn't fly away when the cursor enters the window at first
        if !first_click {
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                first_click = true;
                rl.disable_cursor();
            }
        } else {
            update_camera_angle(&mut player, &mut rl);
        }

        next_tick_in -= rl.get_frame_time();
        while next_tick_in < 0_f32 {
            tick(&mut world, &mut player, &mut rl);
            next_tick_in += TICK_LENGTH;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_BACKSLASH) && first_click { // toggle debug menu
            debug_display = !debug_display;
            if debug_display { open_sound.play() } else { close_sound.play() };
        }

        // FIXME: implement saving menu (waiting on #58)
        // Q for save
        if rl.is_key_pressed(KeyboardKey::KEY_Q) {
            let buf = rmp_serde::to_vec(&world).unwrap();
            fs::write("world.bin", buf);
            // let mut s = flexbuffers::FlexbufferSerializer::new();
            // world.serialize(&mut s).unwrap();
            // fs::write("world.bin", s.view());
            // let serialized = serde_json::to_string(&world).unwrap();
            // fs::write("world.json", serialized);
        }

        // L for load
        if rl.is_key_pressed(KeyboardKey::KEY_L) {
            // FIXME: implement proper error handling
            let bytes = fs::read("world.bin").unwrap();
            world = rmp_serde::from_slice(&bytes).expect("deserialize failed");

            // reset world renderer
            world.mesh_all_chunks(&mut world_renderer);            
            // world = flexbuffers::from_slice::<World>(&bytes).expect("failed to deserialize");
            // let serialized = fs::read("world.json").unwrap();
            // world = serde_json::from_slice::<World>(&serialized).expect("failed to deserialize world");
        }


        rl.draw(&thread, |mut d| {
            d.clear_background(Color::LIGHTBLUE);

            world_renderer.render(&mut d, player.camera);

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
                let p = player.camera.position;
                let mut dir = player.camera.target - player.camera.position;
                dir.normalize();
                let hit = voxel_raycast(&world, p.x, p.y, p.z, dir.x, dir.y, dir.z, Some(100.));
                debug_info += &format!(
                    "Looking at block: {}\n",
                    hit.map_or(
                        String::from("--"),
                        |h| format!(
                            "{:?} - {:.4} {:.4} {:.4}",
                            world.get_block_data(h.x, h.y, h.z),
                            h.x, h.y, h.z
                        )
                    )
                );
                d.draw_text(&debug_info, 20, 20, 16, Color::DARKGREEN);
            }
        });

        if frame % FRAMES_PER_CHUNK == 0 {
            world.generate_next_chunk(&mut world_renderer);
        }
        frame += 1;
    }
}
