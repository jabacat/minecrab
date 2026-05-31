use std::collections::VecDeque;
use std::ptr;

use raylib::prelude::*;

mod player;
mod render;
mod world;

use player::{Player, update_camera_angle, update_camera_position};
use world::collision::{VoxelRaycastHit, voxel_raycast};
use world::generation::World;

use crate::render::mesh_tools;
use crate::render::skybox::{create_skybox_mesh, day_amount};
use crate::render::worldmesh::{WorldRenderer};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

const TICK_LENGTH: f32 = 0.025; // 40 ticks per second

// Generate one chunk every [FRAMES_PER_CHUNK] frames so world generation isn't
// exceedingly laggy at the beginning.
const FRAMES_PER_CHUNK: i32 = 1;

fn tick(world: &mut World, player: &mut Player, rl: &mut RaylibHandle) {
    update_camera_position(player, rl);
    //terrain generation should be in here too, and a lot of other stuff.
    //probably need some kind of (dreaded) GameState object to keep the
    //parameter list from being ridiculous.
}

fn hit_voxel_from_player(player: &mut Player, world: &mut World) -> Option<VoxelRaycastHit> {
    // Return a hit from where the player is looking
    let p = player.camera.position;

    let mut dir = player.camera.target - player.camera.position;
    dir.normalize();

    voxel_raycast(&world, p.x, p.y, p.z, dir.x, dir.y, dir.z, Some(100.))
}

fn update_mesh_on_hit(world: &mut World, h: VoxelRaycastHit) {
    // Update a mesh for a given voxel in hit
    let (cx, cy, cz) = World::get_chunk_coords_of_block(h.x, h.y, h.z);
    world.dispatch_mesh_chunk(cx, cy, cz);
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
    let mut debug_frame_times: VecDeque<f32> = VecDeque::new();
    let mut debug_frame_time_stats: Option<(f32, f32, f32)> = None;

    let mut next_tick_in = 0_f32; // time until we run update_camera()

    let audio_stream = RaylibAudio::init_audio_device().expect("Can init audio.");
    let open_sound = audio_stream
        .new_sound(&"assets/audio/menu-open.ogg")
        .expect("Load sound");
    let close_sound = audio_stream
        .new_sound(&"assets/audio/menu-close.ogg")
        .expect("Load sound");

    let mut t = rl
        .load_texture(&thread, "assets/full-textures.png")
        .expect("Should load 'assets/full-textures.png'.");

    t.gen_texture_mipmaps();

    let texture: ffi::Texture = unsafe { t.unwrap() };

    let mut skybox_mesh: Mesh = create_skybox_mesh();
    let mut skybox_material = rl.load_material_default(&thread);
    let mut skybox_shader = rl.load_shader(
        &thread,
        Some("src/shader/skybox.vert"),
        Some("src/shader/skybox.frag"),
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

            world.ticks += 1;
        }

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

        if rl.is_key_pressed(KeyboardKey::KEY_BACKSLASH) && first_click {
            // toggle debug menu
            debug_display = !debug_display;
            if debug_display {
                open_sound.play()
            } else {
                close_sound.play()
            };
        }

        // Remove block
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && first_click {
            let hit = hit_voxel_from_player(&mut player, &mut world);

            if let Some(h) = hit {
                world.set_block_data(h.x, h.y, h.z, world::blocks::BlockData::AIR);
                update_mesh_on_hit(&mut world, h);
            }
        }

        // Add stone block
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) && first_click {
            let hit = hit_voxel_from_player(&mut player, &mut world);

            if let Some(h) = hit {
                world.set_block_data(
                    h.x + h.normal_x as i64,
                    h.y + h.normal_y as i64,
                    h.z + h.normal_z as i64,
                    world::blocks::BlockData::STONE,
                );
                update_mesh_on_hit(&mut world, h);
            }
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

            let day_amount: f32 = day_amount(world.ticks);
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

            world_renderer.render(&mut d, player.camera);

            let w = d.get_screen_width();
            let h = d.get_screen_height();

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

            if !first_click {
                d.draw_text(
                    "WIP: Click to start updating camera",
                    20,
                    20,
                    16,
                    Color::DARKGREEN,
                );
            }
            if debug_display {
                let mut debug_info = String::new();
                debug_info += &format!(
                    "Camera position: {:.4} {:.4} {:.4}\n",
                    player.camera.position.x, player.camera.position.y, player.camera.position.z
                );
                debug_info += &format!("FPS: {}\n", d.get_fps());
                let p = player.camera.position;
                let mut dir = player.camera.target - player.camera.position;
                dir.normalize();
                let hit = voxel_raycast(&world, p.x, p.y, p.z, dir.x, dir.y, dir.z, Some(100.));
                debug_info += &format!(
                    "Looking at block: {}\n",
                    hit.map_or(String::from("--"), |h| format!(
                        "{:?} - {:.4} {:.4} {:.4}",
                        world.get_block_data(h.x, h.y, h.z),
                        h.x,
                        h.y,
                        h.z
                    ))
                );
                debug_info += &format!("Frames elapsed: {}\n", frame);
                d.draw_text(&debug_info, 20, 20, 16, Color::DARKGREEN);

                // Draw frame time stats
                if let Some((p100, p90, p50)) = debug_frame_time_stats {
                    let (p100, p90, p50) = (
                        (p100 * 1000. * 100.).trunc() / 100.,
                        (p90 * 1000. * 100.).trunc() / 100.,
                        (p50 * 1000. * 100.).trunc() / 100.
                    );
                    d.draw_text(format!("100%: {p100} | 90%: {p90} | 50%: {p50}").as_str(), 20, 100, 12, Color::RED);
                }
                // Draw frame time graph
                for (i, ft) in debug_frame_times.iter().enumerate() {
                    d.draw_rectangle(i as i32 + 20, 100 + 20, 1, (*ft * 1000.) as i32, Color::RED);
                }
                d.draw_line(20, 116 + 20, 320, 116 + 20, Color::GREEN);
            }
        });

        let Vector3 {
            x: px,
            y: py,
            z: pz,
        } = player.camera.position;
        world.generate_surrounding_chunks(px as i64, py as i64, pz as i64, 1);
        if frame % FRAMES_PER_CHUNK == 0 {
            world.poll_chunk_gen_thread(&mut world_renderer);
        }
        frame += 1;
    }
}
