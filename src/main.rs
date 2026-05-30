use std::ptr;

use raylib::prelude::*;

mod player;
mod render;
mod world;
mod game;

use player::{Player, update_camera_angle, update_camera_position};
use world::generation::World;
use world::collision::{voxel_raycast, VoxelRaycastHit};

use game::GameData;

use crate::render::mesh_tools;
use crate::render::pause_menu::PauseMenu;
use crate::render::skybox::{create_skybox_mesh, day_amount};
use crate::render::worldmesh::{WorldRenderer, build_geometry_chunk};

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
        .highdpi()   // disabled since switching to SDL
        .build();

    // Disable exit on esc (default raylib behavior)
    rl.set_exit_key(None);
    
    let texture: ffi::Texture = {
        let mut t = rl
            .load_texture(&thread, "assets/full-textures.png")
            .expect("Should load 'assets/full-textures.png'.");

        t.gen_texture_mipmaps();
        unsafe { t.unwrap() }
    };

    let mut skybox_mesh: Mesh = create_skybox_mesh();
    let mut skybox_material = rl.load_material_default(&thread);
    let mut skybox_shader = rl.load_shader(
        &thread,
        Some("src/shader/skybox.vert"), 
        Some("src/shader/skybox.frag")
    );
    skybox_material.shader = *skybox_shader.as_ref();

    let mut material = rl.load_material_default(&thread);
    let mut block_shader = rl.load_shader(
        &thread, 
        Some("src/shader/block.vert"), 
        Some("src/shader/block.frag")
    );
    material.shader = *block_shader.as_ref();
    
    let maps = material.maps_mut();
    maps[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].texture = texture;

    // don't you dare create a "new" or "init"
    // method for this struct
    let mut gd = GameData {
        rl,
        audio_stream: RaylibAudio::init_audio_device()
            .expect("Can init audio."),

        player: Player::new(),
        world: World::new(),
        world_renderer: WorldRenderer::new(material),
        debug_info: String::new(),
        debug_info_shown: false,
        pause_menu: PauseMenu::new(),
        tick_counter: 0,
    };

    let (rl, world, player) = (&mut gd.rl, &mut gd.world, &mut gd.player);

    let mut next_tick_in = 0_f32;
    let mut frame: i32 = 0;

    while !rl.window_should_close() && !gd.pause_menu.should_quit() {

        next_tick_in -= rl.get_frame_time();
        while next_tick_in < 0_f32 {
            game::tick(&mut gd);
            next_tick_in += TICK_LENGTH;
        }

        rl.draw(&thread, |mut d| {
            d.clear_background(Color::LIGHTBLUE);

            // Skybox

            // So that the skybox doesn't move with the player but still keeps
            // the player's rotation, we create an independent copy of the camera
            // which is shifted back toward the origin always.
            let mut skybox_cam = player.camera.clone();
            skybox_cam.position = Vector3::new(0.0, 0.0, 0.0);
            skybox_cam.target -= player.camera.position;

            let day_amount: f32 = day_amount(frame);
            let skybox_loc = skybox_shader.get_shader_location("dayAmount");
            let block_loc = block_shader.get_shader_location("dayAmount");
            skybox_shader.set_shader_value(skybox_loc, day_amount);
            block_shader.set_shader_value(block_loc, day_amount);

            d.draw_mode3D(skybox_cam, |mut d2, _camera| {
                d2.draw_mesh(&mut skybox_mesh, skybox_material.clone(), Matrix::identity());
            });

            gd.world_renderer.render(&mut d, player.camera);

            let w = d.get_render_width();
            let h = d.get_render_height();

            // Crosshair
            d.draw_line_ex(
                rvec2(w / 2 - 10, h / 2),
                rvec2(w / 2 + 10, h / 2),
                3.0,
                Color::WHITESMOKE,
            );

            d.draw_line_ex(
                rvec2(w / 2, h / 2 - 10),
                rvec2(w / 2, h / 2 + 10),
                3.0,
                Color::WHITESMOKE,
            );

            if gd.debug_info_shown {
                d.draw_text(&gd.debug_info, 20, 20, 16, Color::DARKGREEN);
            }

            // Render pause menu
            pause_menu.render(&mut d);
        });
    }
}
