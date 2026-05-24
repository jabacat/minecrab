use crate::world::generation::generate_chunk;

use raylib::prelude::*;

struct Chunk {

    // The 3D index of the chunk.
    pub x: i64,
    pub y: i64,
    pub z: i64,

    model: Model,

}

impl Chunk {

    pub fn new(x: i64, y: i64, z: i64, rl: &mut RaylibHandle, thread: &RaylibThread, textures: ffi::Texture) -> Chunk {
        let mut c = Chunk {
            x: x,
            y: y,
            z: z,
            model: generate_chunk(rl, &thread, x, y, z)
        };
        c.set_textures(textures);
        c
    }

    pub fn render(&self, handle: &mut RaylibMode3D<RaylibDrawHandle>) {
        handle.draw_model(&self.model, Vector3::zero(), 1., Color::WHITE)
    }

    pub fn set_textures(&mut self, texture: ffi::Texture) {
        let materials = self.model.materials_mut();
        let material = &mut materials[0];
        let maps = material.maps_mut();
        maps[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].texture = texture;
    }

}

const WORLD_RADIUS: i64 = 3;

pub struct World {

    chunks: Vec<Chunk>,

    // For generation purposes, the next chunk to be generated.
    next_gen_x: i64,
    next_gen_y: i64,
    next_gen_z: i64,

}

impl World {

    pub fn new() -> World {
        return World {
            chunks: Vec::new(),
            next_gen_x: -WORLD_RADIUS,
            next_gen_y: -WORLD_RADIUS,
            next_gen_z: -WORLD_RADIUS,
        }
    }

    pub fn render(
        &mut self, d: &mut RaylibDrawHandle, camera: Camera3D
    ) {
        d.draw_mode3D(camera, |mut d2, _camera| {
            for chunk in &self.chunks {
                chunk.render(&mut d2);
            }
        });
    }

    pub fn generate_next_chunk(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, texture: ffi::Texture) {
        // If there is a next chunk to generate, then do so.
        if self.next_gen_x <= WORLD_RADIUS {
            // Generate the chunk and then iterate on.
            self.generate_chunk(self.next_gen_x, self.next_gen_y, self.next_gen_y, rl, thread, texture);
        }
        if self.next_gen_y == WORLD_RADIUS {
            self.next_gen_x += 1;
            self.next_gen_y = 0;
            self.next_gen_z = 0;
        } else if self.next_gen_z == WORLD_RADIUS {
            self.next_gen_y += 1;
            self.next_gen_z = 0;
        } else {
            self.next_gen_z += 1;
        }
    }

    pub fn generate_chunk(&mut self, cx: i64, cy: i64, cz: i64, rl: &mut RaylibHandle, thread: &RaylibThread, texture: ffi::Texture) {
        self.chunks.push(Chunk::new(cx, cy, cz, rl, thread, texture));
    }

}