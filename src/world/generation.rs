use noise::{NoiseFn, SuperSimplex};
use raylib::prelude::*;

use crate::mesh_tools::VecMesh;

const CHUNK_SIZE: i64 = 32;

pub struct BlockData {
    non_void: bool
}

pub struct ChunkData {
    cx: i64, cy: i64, cz: i64,
    block_data: Box<[BlockData]>
}

impl ChunkData {
    pub fn get_block_data(self: &ChunkData, x: i64, y: i64, z: i64) -> BlockData {
        let (lx, ly, lz) = (x - self.cx, y - self.cy, z - self.cz);

        let idx = ly * CHUNK_SIZE * CHUNK_SIZE + lz * CHUNK_SIZE + lx;

        self.block_data[idx];
    }
}

pub fn voxel_generate_terrain(x: i64, y: i64, z: i64) -> BlockData {
    static SSN: std::sync::LazyLock<SuperSimplex> = std::sync::LazyLock::new(|| SuperSimplex::new(42));

    return BlockData {
        non_void: SSN.get([(x as f64 / 16.) , (y as f64 / 16.) , (z as f64 / 16.) ]) > 0.5
    };
}

pub fn chunk_generate_terrain(cx: i64, cy: i64, cz: i64) -> ChunkData {
    let range = -CHUNK_SIZE/2..CHUNK_SIZE/2;
    let voxel_count = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
    let mut block_data = Vec::with_capacity(voxel_count as usize);

    for y in range.clone() {
    for z in range.clone() {
    for x in range.clone() {
        let (x, y, z) = (x + CHUNK_SIZE * cx, y + CHUNK_SIZE * cy, z + CHUNK_SIZE * cz);
        block_data.push(voxel_generate_terrain(x, y, z));
    }}};

    ChunkData {
        cx, cy, cz,
        block_data: block_data.into_boxed_slice()
    }
}


fn voxel_build_geometry(vmesh: &mut VecMesh, x: i64, y: i64, z: i64) {
    if !get_block_data(x, y, z).non_void { return }
    for (dx, dy, dz) in [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ] {
        if get_block_data(x + dx, y + dy, z + dz).non_void {
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


pub fn chunk_build_geometry(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    cx: i64, cy: i64, cz: i64
) -> Model {
    let mut vmesh = VecMesh::new();

    for x in -16..16 {
    for y in -16..16 {
    for z in -16..16 {
        let (x, y, z) = (x + CHUNK_SIZE * cx, y + CHUNK_SIZE * cy, z + CHUNK_SIZE * cz);
        voxel_build_geometry(&mut vmesh, x, y, z);
    }}}


    vmesh.indices.resize(vmesh.vertices.len() / 2, 0);
    for i in 0..vmesh.vertices.len() / 12 {
        // FIXME: these type casts are really ugly; there must be a better way
        // nvm I think this shadowing solution is pretty good
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

    // FIXME: my theory is that vao and vbo should now be initialized
    // unfortunately there seems to be no way to check (?)
    // dbg!(mesh.to_raw().vaoId);
    // dbg!(mesh.to_raw().vboId);

    let model = rl.load_model_from_mesh(thread, unsafe { mesh.make_weak() }).unwrap();

    model
}
