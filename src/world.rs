use noise::{NoiseFn, SuperSimplex};
use raylib::prelude::*;

use crate::mesh_tools::VecMesh;

const CHUNK_SIZE: i64 = 32;
const META_CHUNK_SIZE: i64 = 4;

#[derive(Clone, Copy)]
pub struct BlockData {
    non_void: bool
}

struct Chunk {
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
    voxels: Box<[BlockData]>
}

struct MetaChunk {
    /* absolute meta_chunk coordinates.
     * 1 unit = CHUNK_SIZE * META_CHUNK_SIZE blocks */
    mx: i64, my: i64, mz: i64,

    /* always must have length META_CHUNK_SIZE ^ 3 
    *  ordered the same way as Chunk.voxels */
    chunks: Box<[Chunk]>
}

pub struct World {

    /* in the future...
     *
     * meta_chunks: HashMap<(i64, i64, i64), MetaChunk> */
    meta_chunk: MetaChunk
}

impl Chunk {
    pub fn new(cx: i64, cy: i64, cz: i64) -> Self {
        let voxel_count = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
        let mut voxels = Vec::with_capacity(voxel_count as usize);

        for _ in 0..voxel_count {
            voxels.push(BlockData { non_void: false });
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

impl MetaChunk {
    pub fn new(mx: i64, my: i64, mz: i64) -> Self {
        let mcs = META_CHUNK_SIZE;
        let chunk_count = mcs * mcs * mcs;
        let mut chunks = Vec::with_capacity(chunk_count as usize);

        let r = 0..META_CHUNK_SIZE;
        for ly in r.clone() { for lz in r.clone() { for lx in r.clone() {
            let (cx, cy, cz) = (mx * mcs + lx, my * mcs + ly, mz * mcs + lz);
            chunks.push(Chunk::new(cx, cy, cz));
        }}}

        Self { mx, my, mz, chunks: chunks.into_boxed_slice() }
    }

    pub fn get_block_data(self: &Self, x: i64, y: i64, z: i64) -> BlockData {
        let chunk = &self.chunks[self.get_chunk_idx_of_block(x, y, z)];

        chunk.get_block_data(x, y, z)
    }
    
    pub fn set_block_data(
        self: &mut Self, x: i64, y: i64, z: i64, value: BlockData
    ) {
        let chunk = &mut self.chunks[self.get_chunk_idx_of_block(x, y, z)];

        chunk.set_block_data(x, y, z, value)
    }
    
    fn get_chunk_idx_of_block(self: &Self, x: i64, y: i64, z: i64) -> usize {
        let mcs = META_CHUNK_SIZE;
        let cs = CHUNK_SIZE;

        /* absolute chunk coordinates */
        let (cx, cy, cz) = (x / cs, y / cs, z / cs);
        
        /* local chunk coordinates */
        let (lx, ly, lz) = (
            cx - self.mx * mcs,
            cy - self.my * mcs,
            cz - self.mz * mcs
        );

        for coordinate in [lx, ly, lz] {
            assert!(
                coordinate >= 0 && coordinate < mcs,
                "block coordinate out of bounds: {}, {}, {}",
                x, y, z
            );
        }

        let idx = ly * mcs * mcs + lz * mcs + lx;
        idx as usize
    }
}

impl World {
    pub fn new() -> Self {
        Self { meta_chunk: MetaChunk::new(0, 0, 0) }
    }

    pub fn get_block_data(self: &Self, x: i64, y: i64, z: i64) -> BlockData {
        for coordinate in [x, y, z] {
            if coordinate < 0 || coordinate >= META_CHUNK_SIZE * CHUNK_SIZE {
                /* out of this world! */
                return BlockData { non_void: false }
            }
        }
        self.meta_chunk.get_block_data(x, y, z)
    }

    pub fn set_block_data(
        self: &mut Self, x: i64, y: i64, z: i64, value: BlockData
    ) {
        self.meta_chunk.set_block_data(x, y, z, value);
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
        let r = 0..CHUNK_SIZE;

        for y in r.clone() { for z in r.clone() { for x in r.clone() {
            let (x, y, z) = (
                x + CHUNK_SIZE * cx,
                y + CHUNK_SIZE * cy,
                z + CHUNK_SIZE * cz
            );
            self.generate_terrain_voxel(x, y, z);
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

    pub fn build_geometry_chunk(&mut self, cx: i64, cy: i64, cz: i64) -> Mesh {
        let mut vmesh = VecMesh::new();

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

        mesh

            /*
        // FIXME: my theory is that vao and vbo should now be initialized
        // unfortunately there seems to be no way to check (?)
        // dbg!(mesh.to_raw().vaoId);
        // dbg!(mesh.to_raw().vboId);

        //let model = rl.load_model_from_mesh(thread, unsafe { mesh.make_weak() }).unwrap();

        model
            */
    }
}




