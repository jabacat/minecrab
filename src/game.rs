use raylib::prelude::*;

use crate::*;
use world::generation::*;
use world::blocks::*;
use render::worldmesh::*;
use render::pause_menu::*;
use player::*;

use KeyboardKey::*;
use MouseButton::*;

use world::collision::*;

const TICKS_PER_CHUNK: u32 = 4;

pub struct Sounds<'a> {
    pub menu_open: Sound<'a>,
    pub menu_close: Sound<'a>,
}

pub struct GameData {
    pub paused: bool,

    pub rl: RaylibHandle,

    pub player: Player,
    pub world: World,

    // will be removed
    pub world_renderer: WorldRenderer,

    pub pause_menu: PauseMenu,

    // set every tick to the string to show on the debug screen
    pub debug_info: String,
    pub debug_info_shown: bool,

    pub tick_counter: u32,
    pub frame_counter: u32,

    // in seconds. could use std::time::Duration but i don't see the point.
    pub last_tick_time: f32,
    pub last_frame_time: f32,

    // commented out to stop dead code warning,
    // not sure if we'll need it later or not.
    //
    // pub audio_stream: &'static RaylibAudio,
    pub sounds: &'static Sounds<'static>
}

pub fn tick(gd: &mut GameData) {
    //if gd.rl.is_key_pressed(KEY_ESCAPE) { gd.paused = !gd.paused; }
    gd.pause_menu.update(&mut gd.rl);

    if gd.paused {
        //show pause menu
    } else {
        let (rl, world, player) = (&mut gd.rl, &mut gd.world, &mut gd.player);

        update_camera_position(player, rl);
        update_camera_angle(player, rl);


        if rl.is_key_pressed(KEY_BACKSLASH) {
            gd.debug_info_shown = !gd.debug_info_shown;

            if gd.debug_info_shown { &gd.sounds.menu_open }
            else { &gd.sounds.menu_close }
                .play();
        }
        
        if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) {
            let hit = hit_voxel_from_player(player, world);
            
            if let Some(h) = hit {
                world.set_block_data(h.x, h.y, h.z, BlockData::AIR);
                update_mesh_on_hit(world, h, &mut gd.world_renderer);
            }
        }
        
        // Add stone block
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            let hit = hit_voxel_from_player(player, world);
            
            if let Some(h) = hit {
                world.set_block_data(
                    h.x + h.normal_x as i64,
                    h.y + h.normal_y as i64,
                    h.z + h.normal_z as i64,
                    BlockData::STONE
                );
                update_mesh_on_hit(world, h, &mut gd.world_renderer);
            }
        }

        if gd.tick_counter % TICKS_PER_CHUNK == 0 {
            //world.generate_next_chunk(&mut gd.world_renderer);
            let Vector3 {
                x: px,
                y: py,
                z: pz,
            } = player.camera.position;
            world.generate_surrounding_chunks(&mut gd.world_renderer, px as i64, py as i64, pz as i64, 1);
        }

        gd.debug_info =
            if gd.debug_info_shown { debug_info_fmt(gd) }
            else { String::new() };
    }
}

fn debug_info_fmt(gd: &mut GameData) -> String {
    let hit = hit_voxel_from_player(&mut gd.player, &mut gd.world);
    let looking_at = hit.map_or(
        String::from("--"),
        |h| format!(
            "{:?} - {} {} {}",
            gd.world.get_block_data(h.x, h.y, h.z),
            h.x, h.y, h.z
        )
    );

    let Vector3 {x: cam_x, y: cam_y, z: cam_z} = gd.player.camera.position;
    let fps = "TODO";
    let tick_counter = gd.tick_counter;

    return format!("
        camera position: {cam_x:.4} {cam_y:.4} {cam_z:.4}
        looking at block: {looking_at}
        FPS: {fps}
        tick: {tick_counter}
    ").lines()
        .map(|l| String::from(l.trim_start()) + "\n")
        .collect();
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
