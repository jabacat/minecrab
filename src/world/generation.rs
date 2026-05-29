use noise::{NoiseFn, SuperSimplex};
use raylib::prelude::*;

use std::collections::HashMap;

use crate::render::worldmesh;
use crate::world::blocks::BlockData;

pub const CHUNK_SIZE: i64 = 32;
const WORLD_RADIUS: i64 = 2;

pub struct Chunk {
    /* absolute chunk coordinates
     * 1 unit = CHUNK_SIZE blocks */
    cx: i64, cy: i64, cz: i64,

    /* always must have length CHUNK_SIZE ^ 3
     *
     * ordered by row (x), then by column (z), then by layer (y)!
     *
     * so when iterating, use
     * for (y):
     *   for (z):
     *     for (x): */
    voxels: Box<[BlockData]>,
}

pub struct World {
    next_gen_x: i64,
    next_gen_y: i64,
    next_gen_z: i64,

    pub chunks: HashMap<(i64, i64, i64), Chunk>
}

impl Chunk {
    pub fn new(cx: i64, cy: i64, cz: i64) -> Self {
        let voxel_count = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
        let mut voxels = Vec::with_capacity(voxel_count as usize);

        for _ in 0..voxel_count {
            voxels.push(BlockData::AIR);
        }

        Self { cx, cy, cz, voxels: voxels.into_boxed_slice() }
    }

    pub fn get_block_data(self: &Self, x: i64, y: i64, z: i64) -> BlockData {
        self.voxels[self.get_block_idx(x, y, z)]
    }

    pub fn set_block_data(self: &mut Self, x: i64, y: i64, z: i64, value: BlockData) {
        self.voxels[self.get_block_idx(x, y, z)] = value;
    }
    
    fn get_block_idx(self: &Self, x: i64, y: i64, z: i64) -> usize {
        let (lx, ly, lz) = (
            x - self.cx * CHUNK_SIZE,
            y - self.cy * CHUNK_SIZE,
            z - self.cz * CHUNK_SIZE
        );
        let idx = ly * CHUNK_SIZE * CHUNK_SIZE + lz * CHUNK_SIZE + lx;

        idx as usize
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            next_gen_x: -WORLD_RADIUS,
            next_gen_y: -WORLD_RADIUS,
            next_gen_z: -WORLD_RADIUS,
        }
    }

    pub fn get_chunk_coords_of_block(x: i64, y: i64, z: i64) -> (i64, i64, i64) {
        (
            if x >= 0 { x / CHUNK_SIZE } else { (x+1) / CHUNK_SIZE - 1 },
            if y >= 0 { y / CHUNK_SIZE } else { (y+1) / CHUNK_SIZE - 1 },
            if z >= 0 { z / CHUNK_SIZE } else { (z+1) / CHUNK_SIZE - 1 },
        )
    }

    /* returns BlockData { non_void: false } for blocks in chunks
     * that haven't been generated yet */
    pub fn get_block_data(self: &Self, x: i64, y: i64, z: i64) -> BlockData {
        let (cx, cy, cz) = World::get_chunk_coords_of_block(x, y, z);

        if let Some(chunk) = self.chunks.get(&(cx, cy, cz)) {
            chunk.get_block_data(x, y, z)
        } else {
            BlockData::AIR
        }
    }

    /* panics if used in a chunk that hasn't been generated yet */
    pub fn set_block_data(
        self: &mut Self, x: i64, y: i64, z: i64, value: BlockData
    ) {
        let (cx, cy, cz) = World::get_chunk_coords_of_block(x, y, z);

        if let Some(chunk) = self.chunks.get_mut(&(cx, cy, cz)) {
            chunk.set_block_data(x, y, z, value)
        } else {
            panic!("set block data in a chunk that doesn't exist");
        }
    }

    fn generate_terrain_column(self: &mut Self, x: i64, z: i64, cy: i64) {
        // Generates one column within a chunk
        static SSN: std::sync::LazyLock<SuperSimplex> =
            std::sync::LazyLock::new(|| SuperSimplex::new(42));
        
        // How shallow slopes are. Don't set below 16 or it will error. 
        let noise_scale = 80.;

        let sample_point = [
            ((x as f64 / noise_scale)),
            ((z as f64 / noise_scale))
        ];

        // arbitrary constants, give a height map between 4*12 and 6*12
        let height = ((SSN.get(sample_point) + 5_f64) * 12_f64) as i64; 

        for y in (CHUNK_SIZE * cy)..(CHUNK_SIZE * (cy + 1)) {
            let block_data = if y > height {
                BlockData::AIR
            } else if y == height {
                BlockData::GRASS
            } else if y > height-3 {
                BlockData::DIRT
            } else if y > 4 {
                BlockData::STONE
            } else {
                BlockData::BEDROCK
            };
    
            self.set_block_data(x, y, z, block_data);
        }
    }

    pub fn generate_terrain_chunk(self: &mut Self, cx: i64, cy: i64, cz: i64) {
        let existing_chunk =
            self.chunks.insert((cx, cy, cz), Chunk::new(cx, cy, cz));
        assert!(existing_chunk.is_none());

        let r = 0..CHUNK_SIZE;

        for z in r.clone() { for x in r.clone() {
            let (wx, wz) = (
                x + CHUNK_SIZE * cx,
                z + CHUNK_SIZE * cz
            );
            self.generate_terrain_column(wx, wz, cy);
            }
        };
    }
    
    pub fn generate_next_chunk(self: &mut Self, world_renderer: &mut worldmesh::WorldRenderer) {
        if self.next_gen_x > WORLD_RADIUS {
            // No more chunks left to generate.
            return;
        }

        self.generate_terrain_chunk(
            self.next_gen_x, self.next_gen_y, self.next_gen_z
        );

        world_renderer.add_mesh(
            self.next_gen_x, self.next_gen_y, self.next_gen_z,
            worldmesh::build_geometry_chunk(
                self, self.next_gen_x, self.next_gen_y, self.next_gen_z
            )
        );

        self.next_gen_z += 1;
        if self.next_gen_z > WORLD_RADIUS {
            self.next_gen_y += 1;
            if self.next_gen_y > WORLD_RADIUS {
                self.next_gen_x += 1;
                self.next_gen_y = -WORLD_RADIUS;
            }
            self.next_gen_z = -WORLD_RADIUS;
        }
    }

}




