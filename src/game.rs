use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::time::Instant;

use crate::player::{Player, PlayerData, camera_pos_from_player_pos};
use crate::render::mesh_tools::{MaterialBuilder, draw_mesh2};
use crate::render::skybox;
use crate::render::pause_menu::{PauseMenu, PauseMenuState};
use crate::render::worldmesh::WorldRenderer;
use crate::world::blocks::BlockData;
use crate::world::collision::{VoxelRaycastHit, voxel_raycast};
use crate::world::generation::{ChunkGenThread, World};
use crate::settings::Settings;

use KeyboardKey::*;
use MouseButton::*;

const DBG_FONT_SIZE: i32 = 16;

const TICKRATE: u32 = 40;
const TICK_LENGTH: f32 = 1. / (TICKRATE as f32);

#[derive(Serialize, Deserialize)]
pub struct GameData {
    pub seed: i32,

    pub tick_counter: u64,

    pub player_data: PlayerData,
    pub world: World,
}

pub struct Sounds<'a> {
    pub menu_open: Sound<'a>,
    pub menu_close: Sound<'a>,
}

pub struct GameController {
    // all durations in seconds.
    // could use std::time::Duration but i don't see the point.

    // Persistent game data
    pub game_data: GameData,

    pub paused: bool,
    pub should_quit: bool,

    pub pause_menu: PauseMenu,
    
    pub player: Player,

    // will be removed
    pub world_renderer: WorldRenderer,
    pub cgt: ChunkGenThread,

    pub skybox_mesh: Mesh,
    pub skybox_material: Material,

    // commented out to stop dead code warning,
    // not sure if we'll need it later or not.
    //
    // pub audio_stream: &'static RaylibAudio,
    pub sounds: &'static Sounds<'static>,

    // set every tick to the string to show on the debug screen
    pub debug_text: String,

    pub debug_frame_times: VecDeque<f32>,
    pub debug_info_shown: bool,

    pub frame_counter: u64,
    pub last_tick_time: f32,
    pub next_tick_in: f32,

    // total meaning including time spent waiting, unlike last_tick_time
    // and debug_frame_times which only count the time spent working.
    pub last_frame_total_time: f32,
}

impl GameController {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, gd: GameData) -> Self {
        let skybox_mesh: Mesh = skybox::create_skybox_mesh();

        let skybox_material = MaterialBuilder::init(rl, thread)
            .vert("src/shader/skybox.vert")
            .frag("src/shader/skybox.frag")
            .build();

        let block_material = MaterialBuilder::init(rl, thread)
            .vert("src/shader/block.vert")
            .frag("src/shader/block.frag")
            .map(
                MaterialMapIndex::MATERIAL_MAP_ALBEDO,
                "assets/full-textures.png",
            )
            .build();

        // create a static reference to audio_stream and sounds.
        // not sure if there's a better way to do this.
        let audio_stream = Box::leak(Box::new(
            RaylibAudio::init_audio_device().expect("init audio"),
        ));
        let sounds = Box::leak(Box::new(Sounds {
            menu_open: audio_stream
                .new_sound(&"assets/audio/menu-open.ogg")
                .expect(&"load sound"),
            menu_close: audio_stream
                .new_sound(&"assets/audio/menu-close.ogg")
                .expect(&"load sound"),
        }));

        // FIXME: is rust stupid?
        let seed = gd.seed;
        let player = Player::new(&gd.player_data);

        GameController {
            game_data: gd,
            paused: true,
            should_quit: false,
            pause_menu: PauseMenu::new(),
            player,
            world_renderer: WorldRenderer::new(block_material),
            skybox_mesh,
            skybox_material,
            sounds,
            debug_text: String::new(),
            debug_frame_times: VecDeque::from([0.; 300]),
            debug_info_shown: false,
            frame_counter: 0,
            last_tick_time: 0.,
            next_tick_in: 0.,
            last_frame_total_time: 0.,
            cgt: ChunkGenThread::new(seed as u32),
        }
    }

    pub fn run(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, settings: Settings) {
        while !self.should_quit {
            let frame_start = Instant::now();

            self.next_tick_in -= self.last_frame_total_time;

            if self.next_tick_in < 0_f32 {
                let tick_start = Instant::now();
                self.tick(rl, &settings);
                self.last_tick_time = tick_start.elapsed().as_secs_f32();
                self.next_tick_in += TICK_LENGTH;
            }

            self.update();
            self.render(rl, thread);

            let frame_compute_time = frame_start.elapsed().as_secs_f32();
            self.debug_frame_times.push_back(frame_compute_time);
            while self.debug_frame_times.len() > 300 {
                self.debug_frame_times.pop_front();
            }

            unsafe {
                raylib::ffi::SwapScreenBuffer();
            }

            self.frame_counter += 1;
            self.last_frame_total_time = frame_start.elapsed().as_secs_f32();
        }
    }

    /// Updates that run at a fixed tick rate
    pub fn tick(&mut self, rl: &mut RaylibHandle, settings: &Settings) {
        unsafe {
            raylib::ffi::PollInputEvents();
        }

        self.should_quit |= rl.window_should_close();

        // Loading and saving
        // XXX: This is at the top so that the loading screens can render once
        // before the load/save (which takes a while) actually happens
        // Save
        if self.pause_menu.should_save() {
            let buf = rmp_serde::to_vec(&self.game_data).expect("serialize failed");
            fs::write("world.bin", buf).expect("writing save to file failed");

            // reset pause menu state
            self.pause_menu.set_state(rl, PauseMenuState::Paused);
        }

        // Load
        if self.pause_menu.should_load() {
            // FIXME: implement proper error handling
            let bytes = fs::read("world.bin").expect("reading save from file failed");
            self.game_data = rmp_serde::from_slice(&bytes).expect("deserialize failed");

            // Terminate old chunk gen thread and start new chunk gen thread
            let new_cgt = ChunkGenThread::new(self.game_data.seed as u32);
            std::mem::replace(&mut self.cgt, new_cgt).join().unwrap();

            // reset world renderer
            self.mesh_all_chunks();

            // reset player camera
            self.player.reset_player(&self.game_data.player_data);

            // reset pause menu state
            self.pause_menu.set_state(rl, PauseMenuState::Paused);
        }

        if !self.paused {
            self.game_data.tick_counter += 1;

            self.player
                .process_tick(&mut self.game_data.player_data, rl, &self.game_data.world);

            if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) {
                let hit = self.hit_voxel_from_player();

                if let Some(h) = hit {
                    self.game_data
                        .world
                        .set_block_data(h.x, h.y, h.z, BlockData::AIR);
                    self.update_mesh_on_hit(h);
                }
            }

            // Add stone block
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
                let hit = self.hit_voxel_from_player();

                if let Some(h) = hit {
                    self.game_data.world.set_block_data(
                        h.x + h.normal_x as i64,
                        h.y + h.normal_y as i64,
                        h.z + h.normal_z as i64,
                        BlockData::STONE,
                    );
                    self.update_mesh_on_hit(h);
                }
            }
        }

        // Update pause menu
        self.pause_menu.update(rl);
        self.paused = !self.pause_menu.is_running();
        self.should_quit |= self.pause_menu.should_quit();

        // Progressive chunk generation
        self.generate_surrounding_chunks(settings.render_distance);

        // Poll for generated chunks
        let chunk_gen_result = self.cgt.poll();
        if let Some(result) = chunk_gen_result {
            if let Some(chunk) = result.3 {
                self.game_data
                    .world
                    .chunks
                    .insert((result.0, result.1, result.2), chunk);
            }

            let mut mesh = result.4.to_mesh();
            unsafe { mesh.upload(false) };
            self.world_renderer
                .add_mesh(result.0, result.1, result.2, mesh);
        }

        if rl.is_key_pressed(KEY_BACKSLASH) {
            self.debug_info_shown = !self.debug_info_shown;

            if self.debug_info_shown {
                &self.sounds.menu_open
            } else {
                &self.sounds.menu_close
            }
            .play();
        }

        if self.debug_info_shown {
            self.debug_text = self.debug_info_fmt();
        }
    }

    /// Updates that run every frame
    pub fn update(&mut self) {
        self.update_camera();
    }

    pub fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::LIGHTBLUE);

        // Make sure to get the render order right!

        // Skybox
        self.render_skybox(&mut d);

        // World
        self.world_renderer.render(&mut d, self.player.camera);

        // Crosshair
        GameController::draw_crosshair(&mut d);
        
        // Pause menu
        if self.paused {
            self.pause_menu.render(&mut d);
        }

        // Debug Info
        self.render_debug(&mut d);

    }

    pub fn cleanup(self) {
        self.cgt.join().unwrap();
    }

    fn update_camera(&mut self) {
        // on a scale of zero to one, how close are we to the next tick.
        let interp = 1. - (self.next_tick_in / TICK_LENGTH).clamp(0., 1.);
        if !self.paused {
            self.player
                .update_camera(&mut self.game_data.player_data, interp, &self.game_data.world);
        }
    }

    fn render_skybox(&mut self, d: &mut RaylibDrawHandle) {
        // So that the skybox doesn't move with the player but still keeps
        // the player's rotation, we create an independent copy of the camera
        // which is shifted back toward the origin always.
        let mut skybox_cam = self.player.camera.clone();
        skybox_cam.position = Vector3::new(0.0, 0.0, 0.0);
        skybox_cam.target -= self.game_data.player_data.pos;

        let day_amount: f32 = skybox::day_amount(self.game_data.tick_counter);
        let skybox_loc = self
            .skybox_material
            .shader()
            .get_shader_location("dayAmount");
        let block_loc = self
            .world_renderer
            .material
            .shader()
            .get_shader_location("dayAmount");
        self.skybox_material
            .shader_mut()
            .set_shader_value(skybox_loc, day_amount);
        self.world_renderer
            .material
            .shader_mut()
            .set_shader_value(block_loc, day_amount);

        d.draw_mode3D(skybox_cam, |d2, _camera| {
            draw_mesh2(
                &d2,
                &mut self.skybox_mesh,
                &self.skybox_material,
                Matrix::identity(),
            );
        });
    }

    fn render_debug(&self, d: &mut RaylibDrawHandle) {
        if self.debug_info_shown {
            d.draw_text(&self.debug_text, 20, 20, DBG_FONT_SIZE, Color::BLACK);
            let text = if self.debug_frame_times.len() >= 300 {
                let mut sorted_ft = self.debug_frame_times.iter().collect::<Vec<_>>();
                sorted_ft.sort_by(|a, b| f32::total_cmp(*b, *a));
                let p100 = *sorted_ft[0] * 1000.;
                let p99 = *sorted_ft[2] * 1000.;
                let p90 = *sorted_ft[29] * 1000.;
                let p50 = *sorted_ft[149] * 1000.;
                &format!("frame 100%: {p100:.2} | 99%: {p99:.2} | 90%: {p90:.2} | 50%: {p50:.2}")
            } else {
                "waiting for enough frames..."
            };

            let y = (self.debug_text.lines().count() as i32) * DBG_FONT_SIZE + 20;
            d.draw_text(text, 20, y, 12, Color::RED);

            // Draw frame time graph
            for (i, ft) in self.debug_frame_times.iter().enumerate() {
                d.draw_rectangle(i as i32 + 20, y + 20, 1, (*ft * 1000.) as i32, Color::RED);
            }
            d.draw_line(20, y + 36, 320, y + 36, Color::DARKGREEN);
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

    fn generate_surrounding_chunks(&mut self, render_distance: i64) {
        let Vector3 {
            x: px,
            y: py,
            z: pz,
        } = self.game_data.player_data.pos;
        let (cx, cy, cz) = World::get_chunk_coords_of_block(px as i64, py as i64, pz as i64);

        // Iterate from -radius to radius from lowest magnitude
        // Probably not the most efficient way to to do this
        let mut delta = (-render_distance..=render_distance).collect::<Vec<i64>>();
        delta.sort_by_key(|i| i.abs());

        for dx in &delta {
            for dy in &delta {
                for dz in &delta {
                    let (cx, cy, cz) = (cx + dx, cy + dy, cz + dz);

                    if !self.game_data.world.chunks.contains_key(&(cx, cy, cz)) {
                        self.cgt.dispatch_chunk_gen(cx, cy, cz);
                    }
                }
            }
        }
    }

    // FIXME: inline
    fn mesh_all_chunks(&mut self) {
        self.world_renderer.clear_meshes();
        for (k, v) in &self.game_data.world.chunks {
            self.cgt.dispatch_mesh_chunk(v, k.0, k.1, k.2);
        }
    }

    fn debug_info_fmt(&mut self) -> String {
        let hit = self.hit_voxel_from_player();
        let looking_at = hit.map_or(String::from("--"), |h| {
            format!(
                "{:?} - {} {} {}",
                self.game_data.world.get_block_data(h.x, h.y, h.z),
                h.x,
                h.y,
                h.z
            )
        });

        let Vector3 {
            x: cam_x,
            y: cam_y,
            z: cam_z,
        } = self.game_data.player_data.pos;
        let fps = 1. / self.last_frame_total_time;

        return format!(
            "
            camera position: {cam_x:.4} {cam_y:.4} {cam_z:.4}
            looking at block: {looking_at}
            FPS: {fps}
        "
        )
        .lines()
        .map(|l| String::from(l.trim_start()) + "\n")
        .collect();
    }

    fn hit_voxel_from_player(&self) -> Option<VoxelRaycastHit> {
        // Return a hit from where the player is looking
        let p = camera_pos_from_player_pos(self.game_data.player_data.pos);

        let mut dir = self.game_data.player_data.fwd - self.game_data.player_data.pos;
        dir.normalize();

        voxel_raycast(
            &self.game_data.world,
            p.x,
            p.y,
            p.z,
            dir.x,
            dir.y,
            dir.z,
            Some(100.),
        )
    }

    fn update_mesh_on_hit(&self, h: VoxelRaycastHit) {
        // Update a mesh for a given voxel in hit
        let (cx, cy, cz) = World::get_chunk_coords_of_block(h.x, h.y, h.z);
        let chunk = self
            .game_data
            .world
            .chunks
            .get(&(cx, cy, cz))
            .expect("hit a chunk that doesn't exist");
        self.cgt.dispatch_mesh_chunk(chunk, cx, cy, cz);
    }
}
