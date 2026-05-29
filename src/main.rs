use raylib::prelude::*;

mod player;
mod render;
mod world;

use player::{Player, update_camera_angle, update_camera_position};
use world::generation::World;
use world::collision::{voxel_raycast, VoxelRaycastHit};

use crate::render::mesh_tools;
use crate::render::pause_menu::PauseMenu;
use crate::render::worldmesh::{WorldRenderer, build_geometry_chunk};

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

fn hit_voxel_from_player(player: &mut Player, world: &mut World) -> Option<VoxelRaycastHit> {
    // Return a hit from where the player is looking
    let p = player.camera.position;

    let mut dir = player.camera.target - player.camera.position;
    dir.normalize();

    voxel_raycast(&world, p.x, p.y, p.z, dir.x, dir.y, dir.z, Some(100.))
}

fn update_mesh_on_hit(world: &mut World, h: VoxelRaycastHit, world_renderer: &mut WorldRenderer) {
    // Update a mesh for a given voxel in hit
    let (cx, cy, cz) = World::get_chunk_coords_of_block(h.x, h.y, h.z);
    let mesh = build_geometry_chunk(world, cx, cy, cz);

    world_renderer.add_mesh(cx, cy, cz, mesh);
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Minecrab")
        .vsync()
        .highdpi()   // disabled since switching to SDL
        .build();

    // Disable exit on esc (default raylib behavior)
    rl.set_exit_key(Some(KeyboardKey::KEY_NULL));

    let mut player = Player::new();

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

    let mut window_should_close = false;
    let mut pause_menu = PauseMenu::new();
    let mut debug_display = false; // toggle

    // DEBUG
    rl.set_target_fps(1);

    while !window_should_close {
        window_should_close |= rl.window_should_close();

        pause_menu.update(&mut rl);
        window_should_close |= pause_menu.should_quit();

        if pause_menu.is_running() {
            update_camera_angle(&mut player, &mut rl);

            next_tick_in -= rl.get_frame_time();
            while next_tick_in < 0_f32 {
                tick(&mut world, &mut player, &mut rl);
                next_tick_in += TICK_LENGTH;
            }
            
            if rl.is_key_pressed(KeyboardKey::KEY_BACKSLASH) { // toggle debug menu
                debug_display = !debug_display;
                if debug_display { open_sound.play() } else { close_sound.play() };
            }
            
            // Remove block
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                let hit = hit_voxel_from_player(&mut player, &mut world);
                
                if let Some(h) = hit {
                    world.set_block_data(h.x, h.y, h.z, world::blocks::BlockData::AIR);
                    update_mesh_on_hit(&mut world, h, &mut world_renderer);
                }
            }
            
            // Add stone block
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
                let hit = hit_voxel_from_player(&mut player, &mut world);
                
                if let Some(h) = hit {
                    world.set_block_data(
                        h.x + h.normal_x as i64,
                        h.y + h.normal_y as i64,
                        h.z + h.normal_z as i64,
                        world::blocks::BlockData::STONE
                    );
                    update_mesh_on_hit(&mut world, h, &mut world_renderer);
                }
            }
        }

        rl.draw(&thread, |mut d| {
            d.clear_background(Color::LIGHTBLUE);

            world_renderer.render(&mut d, player.camera);

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

            // Render pause menu
            pause_menu.render(&mut d);
        });

        if frame % FRAMES_PER_CHUNK == 0 {
            world.generate_next_chunk(&mut world_renderer);
        }
        frame += 1;
    }
}
