use noise::{NoiseFn, SuperSimplex};
use raylib::prelude::*;

use std::collections::HashMap;

use crate::mesh_tools::VecMesh;

const CHUNK_SIZE: i64 = 32;
const WORLD_RADIUS: i64 = 2;

#[derive(Clone, Copy)]
pub struct BlockData {
    non_void: bool
}

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

    pub mesh: Option<Mesh>,
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
            voxels.push(BlockData { non_void: false });
        }

        Self { cx, cy, cz, voxels: voxels.into_boxed_slice(), mesh: None }
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
            BlockData { non_void: false }
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

    fn generate_terrain_voxel(self: &mut Self, x: i64, y: i64, z: i64) {
        static SSN: std::sync::LazyLock<SuperSimplex> =
            std::sync::LazyLock::new(|| SuperSimplex::new(42));

        let noise_scale = 16.;

        let sample_point = [
            (x as f64 / noise_scale),
            (y as f64 / noise_scale),
            (z as f64 / noise_scale)
        ];

        let block_data = BlockData {
            non_void: SSN.get(sample_point) > 0.5
        };

        self.set_block_data(x, y, z, block_data);
    }

    pub fn generate_terrain_chunk(self: &mut Self, cx: i64, cy: i64, cz: i64) {
        let existing_chunk =
            self.chunks.insert((cx, cy, cz), Chunk::new(cx, cy, cz));
        assert!(existing_chunk.is_none());

        let r = 0..CHUNK_SIZE;

        for y in r.clone() { for z in r.clone() { for x in r.clone() {
            let (wx, wy, wz) = (
                /* FIXME: this is definitely broken on negative numbers
                 * . or something around here is.
                 * i'm too tired to debug this, gotta wake up early tomorrow
                 */
                x + CHUNK_SIZE * cx,
                y + CHUNK_SIZE * cy,
                z + CHUNK_SIZE * cz
            );
            self.generate_terrain_voxel(wx, wy, wz);
        }}};
    }
 
    fn build_geometry_voxel(
        self: &mut Self, vmesh: &mut VecMesh, x: i64, y: i64, z: i64
    ) {
        if !self.get_block_data(x, y, z).non_void { return }
        for (dx, dy, dz) in [
            (-1, 0, 0),
            (1, 0, 0),
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
        ] {
            if self.get_block_data(x + dx, y + dy, z + dz).non_void {
                continue;
            }

            // if we've hit an air block, then we have a visible block face
            // add the vertices accordingly
            let (x, y, z) = (x as f32, y as f32, z as f32);
            if dx == 1 {
                // FIXME: how do I stop this from getting formatted
                // #[rustfmt::skip]
                vmesh.vertices.extend_from_slice(&[
                    x + 1., y, z,
                    x + 1., y + 1., z,
                    x + 1., y + 1., z + 1.,
                    x + 1., y, z + 1.,
                ]);
                vmesh.texcoords.extend_from_slice(&[
                    0.1, 0.0,
                    0.1, 0.1,
                    0.0, 0.1,
                    0.0, 0.0,
                ]);
            } else if dx == -1 {
                vmesh.vertices.extend_from_slice(&[
                    x, y, z,
                    x, y, z + 1.,
                    x, y + 1., z + 1.,
                    x, y + 1., z,
                ]);
                vmesh.texcoords.extend_from_slice(&[
                    0.0, 0.0,
                    0.1, 0.0,
                    0.1, 0.1,
                    0.0, 0.1,
                ]);
            } else if dy == 1 {
                vmesh.vertices.extend_from_slice(&[
                    x, y + 1., z,
                    x, y + 1., z + 1.,
                    x + 1., y + 1., z + 1.,
                    x + 1., y + 1., z,
                ]);
                vmesh.texcoords.extend_from_slice(&[
                    0.0, 0.1,
                    0.0, 0.0,
                    0.1, 0.0,
                    0.1, 0.1,
                ]);
            } else if dy == -1 {
                vmesh.vertices.extend_from_slice(&[
                    x, y, z,
                    x + 1., y, z,
                    x + 1., y, z + 1.,
                    x, y, z + 1.,
                ]);
                vmesh.texcoords.extend_from_slice(&[
                    0.0, 0.0,
                    0.1, 0.0,
                    0.1, 0.1,
                    0.0, 0.1,
                ]);
            } else if dz == 1 {
                vmesh.vertices.extend_from_slice(&[
                    x, y, z + 1.0,
                    x + 1., y, z + 1.0,
                    x + 1., y + 1., z + 1.0,
                    x, y + 1., z + 1.0,
                ]);
                vmesh.texcoords.extend_from_slice(&[
                    0.0, 0.0,
                    0.1, 0.0,
                    0.1, 0.1,
                    0.0, 0.1,
                ]);
            } else if dz == -1 {
                vmesh.vertices.extend_from_slice(&[
                    x, y, z,
                    x, y + 1., z,
                    x + 1., y + 1., z,
                    x + 1., y, z,
                ]);
                vmesh.texcoords.extend_from_slice(&[
                    0.1, 0.0,
                    0.1, 0.1,
                    0.0, 0.1,
                    0.0, 0.0,
                ]);
            }

            // dx, dy, dz give us the normals for this face
            let (dx, dy, dz) = (dx as f32, dy as f32, dz as f32);
            vmesh.normals.extend_from_slice(&[
                dx, dy, dz,
                dx, dy, dz,
                dx, dy, dz,
                dx, dy, dz,
            ]);
        }
    }

    pub fn build_geometry_chunk(&mut self, cx: i64, cy: i64, cz: i64) {
        let mut vmesh = VecMesh::new();
        
        assert!(self.chunks.contains_key(&(cx, cy, cz)));

        let r = 0..CHUNK_SIZE;

        for y in r.clone() { for z in r.clone() { for x in r.clone() {
            let (x, y, z) = (
                x + CHUNK_SIZE * cx,
                y + CHUNK_SIZE * cy,
                z + CHUNK_SIZE * cz
            );
            self.build_geometry_voxel(&mut vmesh, x, y, z);
        }}}


        vmesh.indices.resize(vmesh.vertices.len() / 2, 0);
        for i in 0..vmesh.vertices.len() / 12 {
            let k = i as u16;
            vmesh.indices[6 * i] = 4 * k;
            vmesh.indices[6 * i + 1] = 4 * k + 1;
            vmesh.indices[6 * i + 2] = 4 * k + 2;
            vmesh.indices[6 * i + 3] = 4 * k;
            vmesh.indices[6 * i + 4] = 4 * k + 2;
            vmesh.indices[6 * i + 5] = 4 * k + 3;
        }

        let mut mesh = vmesh.to_mesh();
        unsafe { mesh.upload(false) };

        let chunk = self.chunks.get_mut(&(cx, cy, cz)).unwrap();

        chunk.mesh = Some(mesh);
    }
    
    pub fn generate_next_chunk(self: &mut Self) {
        if self.next_gen_x > WORLD_RADIUS {
            // No more chunks left to generate.
            return;
        }

        self.generate_terrain_chunk(
            self.next_gen_x, self.next_gen_y, self.next_gen_z
        );

        self.build_geometry_chunk(
            self.next_gen_x, self.next_gen_y, self.next_gen_z
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




