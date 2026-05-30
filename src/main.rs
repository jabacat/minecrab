use std::collections::VecDeque;

use raylib::prelude::*;

mod player;
mod render;
mod world;
mod game;

use player::Player;
use world::generation::World;

use game::*;

use render::{mesh_tools, pause_menu::PauseMenu, skybox};
use render::worldmesh::WorldRenderer;

use std::time::Instant;

const DBG_FONT_SIZE: i32 = 16;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const TICKRATE: u32 = 40;
const TICK_LENGTH: f32 = 1./(TICKRATE as f32);

//struct RenderData {
//    pub skybox_mesh: Mesh,
//    
//    pub skybox_material: WeakMaterial,
//    pub block_material: WeakMaterial
//}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Minecrab")
        .vsync()
        .highdpi()   // disabled since switching to SDL
        .build();

    // Disable exit on esc (default raylib behavior)
    rl.set_exit_key(None);
    rl.disable_cursor();
    
    let texture: ffi::Texture = {
        let mut t = rl
            .load_texture(&thread, "assets/full-textures.png")
            .expect("Should load 'assets/full-textures.png'.");

        t.gen_texture_mipmaps();
        unsafe { t.unwrap() }
    };
    
    let mut debug_frame_times: VecDeque<f32> = VecDeque::new();
    let mut debug_frame_time_stats: Option<(f32, f32, f32)> = None;

    let mut skybox_mesh: Mesh = skybox::create_mesh();
    let mut skybox_material: WeakMaterial = rl.load_material_default(&thread);
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
        Some("src/shader/block.frag"),
    );
    material.shader = *block_shader.as_ref();

    let maps = material.maps_mut();
    maps[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].texture = texture;

    // create a static reference to audio_stream and sounds.
    // not sure if there's a better way to do this.
    let audio_stream = Box::leak(Box::new(
        RaylibAudio::init_audio_device().expect("init audio")
    ));
    let sounds = Box::leak(Box::new(
        Sounds {
            menu_open: audio_stream
                .new_sound(&"assets/audio/menu-open.ogg")
                .expect(&"load sound"),
            menu_close: audio_stream
                .new_sound(&"assets/audio/menu-close.ogg")
                .expect(&"load sound"),
        }
    ));

    // don't you dare create a "new"
    // or "init" method for this struct
    let mut gd = GameData {
        rl,
        sounds,
        player: Player::new(),
        world: World::new(),
        world_renderer: WorldRenderer::new(material),
        debug_info: String::new(),
        debug_info_shown: false,
        paused: false,
        pause_menu: PauseMenu::new(),
        tick_counter: 0,
        frame_counter: 0,
        last_tick_time: 0.,
        last_frame_time: 0.,
    };

    let mut next_tick_in = 0_f32;

    while !gd.rl.window_should_close() && !gd.pause_menu.should_quit() {

        gd.last_frame_time = gd.rl.get_frame_time();
        next_tick_in -= gd.last_frame_time;
        while next_tick_in < 0_f32 {
            let tick_start = Instant::now();
            game::tick(&mut gd);
            gd.tick_counter += 1;
            gd.last_tick_time = tick_start.elapsed().as_secs_f32();
            next_tick_in += TICK_LENGTH;
        }

        let (rl, player) = (&mut gd.rl, &mut gd.player);

        // Debug: add frame times to frame time graph
        if debug_frame_times.len() > 300 {
            debug_frame_times.pop_front();

            // compute some basic stats
            // technically this does mean we are one frame delayed
            // but it saves me from writing another if statement
            // FIXME: this looks like a lot of computation but I don't think
            // it's actually costing us any performance
            let mut sorted_ft = debug_frame_times.iter().collect::<Vec<_>>();
            sorted_ft.sort_by(|a, b| f32::total_cmp(*b, *a));
            debug_frame_time_stats = Some((*sorted_ft[2], *sorted_ft[29], *sorted_ft[149]));
        }
        debug_frame_times.push_back(rl.get_frame_time());


        rl.draw(&thread, |mut d| {
            d.clear_background(Color::LIGHTBLUE);

            // Skybox

            // So that the skybox doesn't move with the player but still keeps
            // the player's rotation, we create an independent copy of the camera
            // which is shifted back toward the origin always.
            let mut skybox_cam = player.camera.clone();
            skybox_cam.position = Vector3::new(0.0, 0.0, 0.0);
            skybox_cam.target -= player.camera.position;

            let day_amount: f32 = skybox::get_sky_brightness(gd.tick_counter);
            let skybox_loc = skybox_shader.get_shader_location("dayAmount");
            let block_loc = block_shader.get_shader_location("dayAmount");
            skybox_shader.set_shader_value(skybox_loc, day_amount);
            block_shader.set_shader_value(block_loc, day_amount);

            d.draw_mode3D(skybox_cam, |mut d2, _camera| {
                d2.draw_mesh(
                    &mut skybox_mesh,
                    skybox_material.clone(),
                    Matrix::identity(),
                );
            });

            gd.world_renderer.render(&mut d, player.camera);

            draw_crosshair(&mut d);

            if gd.debug_info_shown {
                d.draw_text(&gd.debug_info, 20, 20, DBG_FONT_SIZE, Color::BLACK);

                let frame_graph_y = (gd.debug_info.lines().count() as i32) * DBG_FONT_SIZE + 20;
                if let Some((p100, p90, p50)) = debug_frame_time_stats {
                    let (p100, p90, p50) = (
                        (p100 * 1000. * 100.).trunc() / 100.,
                        (p90 * 1000. * 100.).trunc() / 100.,
                        (p50 * 1000. * 100.).trunc() / 100.
                    );
                    let text = format!("100%: {p100} | 90%: {p90} | 50%: {p50}");
                    d.draw_text(&text, 20, frame_graph_y, 12, Color::RED);
                }
                // Draw frame time graph
                for (i, ft) in debug_frame_times.iter().enumerate() {
                    d.draw_rectangle(i as i32 + 20, frame_graph_y + 20, 1, (*ft * 1000.) as i32, Color::RED);
                }
                d.draw_line(20, frame_graph_y + 36, 320, frame_graph_y + 36, Color::DARKGREEN);
            }

            // Render pause menu
            gd.pause_menu.render(&mut d);
        });

        gd.frame_counter += 1;
    }
}

fn draw_crosshair(d: &mut RaylibDrawHandle) {
    let w = d.get_render_width();
    let h = d.get_render_height();
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
}
