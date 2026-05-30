use raylib::prelude::*;
use std::collections::HashMap;

use crate::mesh_tools::VecMesh;
use crate::world::blocks::{BlockData, BlockTextureCoordinates};
use crate::world::generation::{CHUNK_SIZE, Chunk};

pub fn remote_build_geometry_chunk(chunk: &Chunk, cx: i64, cy: i64, cz: i64) -> VecMesh {
    let mut vmesh = VecMesh::new();

    let r = 0..CHUNK_SIZE;

    for y in r.clone() { for z in r.clone() { for x in r.clone() {
        let (x, y, z) = (
            x + CHUNK_SIZE * cx,
            y + CHUNK_SIZE * cy,
            z + CHUNK_SIZE * cz
        );
        remote_build_geometry_voxel(chunk, &mut vmesh, x, y, z);
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

    vmesh
}

pub fn remote_build_geometry_voxel(
    chunk: &Chunk, vmesh: &mut VecMesh, x: i64, y: i64, z: i64
) {
    let block_type = chunk.get_block_data(x, y, z);
    if block_type == BlockData::AIR { return }
    let base = BlockTextureCoordinates::new(block_type);
    for (dx, dy, dz) in [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ] {
        if chunk.is_within_bounds(x + dx, y + dy, z + dz) && chunk.get_block_data(x + dx, y + dy, z + dz) != BlockData::AIR {
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
                base.right.0 + 0.1, base.right.1 + 0.1,
                base.right.0 + 0.1, base.right.1 + 0.0,
                base.right.0 + 0.0, base.right.1 + 0.0,
                base.right.0 + 0.0, base.right.1 + 0.1,
            ]);
        } else if dx == -1 {
            vmesh.vertices.extend_from_slice(&[
                x, y, z,
                x, y, z + 1.,
                x, y + 1., z + 1.,
                x, y + 1., z,
            ]);
            vmesh.texcoords.extend_from_slice(&[
                base.left.0 + 0.0, base.left.1 + 0.1,
                base.left.0 + 0.1, base.left.1 + 0.1,
                base.left.0 + 0.1, base.left.1 + 0.0,
                base.left.0 + 0.0, base.left.1 + 0.0,
            ]);
        } else if dy == 1 {
            vmesh.vertices.extend_from_slice(&[
                x, y + 1., z,
                x, y + 1., z + 1.,
                x + 1., y + 1., z + 1.,
                x + 1., y + 1., z,
            ]);
            vmesh.texcoords.extend_from_slice(&[
                base.top.0 + 0.0, base.top.1 + 0.1,
                base.top.0 + 0.0, base.top.1 + 0.0,
                base.top.0 + 0.1, base.top.1 + 0.0,
                base.top.0 + 0.1, base.top.1 + 0.1,
            ]);
        } else if dy == -1 {
            vmesh.vertices.extend_from_slice(&[
                x, y, z,
                x + 1., y, z,
                x + 1., y, z + 1.,
                x, y, z + 1.,
            ]);
            vmesh.texcoords.extend_from_slice(&[
                base.bottom.0 + 0.0, base.bottom.1 + 0.0,
                base.bottom.0 + 0.1, base.bottom.1 + 0.0,
                base.bottom.0 + 0.1, base.bottom.1 + 0.1,
                base.bottom.0 + 0.0, base.bottom.1 + 0.1,
            ]);
        } else if dz == 1 {
            vmesh.vertices.extend_from_slice(&[
                x, y, z + 1.0,
                x + 1., y, z + 1.0,
                x + 1., y + 1., z + 1.0,
                x, y + 1., z + 1.0,
            ]);
            vmesh.texcoords.extend_from_slice(&[
                base.front.0 + 0.0, base.front.1 + 0.1,
                base.front.0 + 0.1, base.front.1 + 0.1,
                base.front.0 + 0.1, base.front.1 + 0.0,
                base.front.0 + 0.0, base.front.1 + 0.0,
            ]);
        } else if dz == -1 {
            vmesh.vertices.extend_from_slice(&[
                x, y, z,
                x, y + 1., z,
                x + 1., y + 1., z,
                x + 1., y, z,
            ]);
            vmesh.texcoords.extend_from_slice(&[
                base.back.0 + 0.1, base.back.1 + 0.1,
                base.back.0 + 0.1, base.back.1 + 0.0,
                base.back.0 + 0.0, base.back.1 + 0.0,
                base.back.0 + 0.0, base.back.1 + 0.1,
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

pub struct WorldRenderer {
    // Meshes are stored with a key of their chunk index
    chunk_meshes: HashMap<(i64, i64, i64), Mesh>,
    material: WeakMaterial,
}

impl WorldRenderer {

    pub fn new(material: WeakMaterial) -> WorldRenderer {
        WorldRenderer {
            chunk_meshes: HashMap::new(),
            material: material,
        }
    }

    pub fn add_mesh(&mut self, cx: i64, cy: i64, cz: i64, mesh: Mesh) {
        // Insert will overwrite the mesh based on their chunk index
        self.chunk_meshes.insert((cx, cy, cz), mesh);
    }

    pub fn render(&mut self, d: &mut RaylibDrawHandle, camera: Camera3D) {
        d.draw_mode3D(camera, |mut d2, _camera| {
            for (_, mesh) in &self.chunk_meshes {
                d2.draw_mesh(mesh, self.material.clone(), Matrix::identity());
            }
        });
    }

}
