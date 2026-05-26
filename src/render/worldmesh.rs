use raylib::prelude::*;

use crate::mesh_tools::VecMesh;
use crate::world::generation::{Chunk, CHUNK_SIZE, World};

pub fn build_geometry_voxel(
    world: &mut World, vmesh: &mut VecMesh, x: i64, y: i64, z: i64
) {
    if !world.get_block_data(x, y, z).non_void { return }
    for (dx, dy, dz) in [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ] {
        if world.get_block_data(x + dx, y + dy, z + dz).non_void {
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

pub fn build_geometry_chunk(world: &mut World, cx: i64, cy: i64, cz: i64) -> Mesh {
    let mut vmesh = VecMesh::new();
    
    assert!(world.chunks.contains_key(&(cx, cy, cz)));

    let r = 0..CHUNK_SIZE;

    for y in r.clone() { for z in r.clone() { for x in r.clone() {
        let (x, y, z) = (
            x + CHUNK_SIZE * cx,
            y + CHUNK_SIZE * cy,
            z + CHUNK_SIZE * cz
        );
        build_geometry_voxel(world, &mut vmesh, x, y, z);
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

    return mesh
}

pub struct WorldRenderer {
    // Right now, all we need is a list of meshes to render.
    // IN THE FUTURE, this should be a map of some sort.
    chunk_meshes: Vec<Mesh>,
    material: WeakMaterial,
}

impl WorldRenderer {

    pub fn new(material: WeakMaterial) -> WorldRenderer {
        WorldRenderer {
            chunk_meshes: Vec::new(),
            material: material,
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.chunk_meshes.push(mesh)
    }

    pub fn render(&mut self, d: &mut RaylibDrawHandle, camera: Camera3D) {
        d.draw_mode3D(camera, |mut d2, _camera| {
            for mesh in &self.chunk_meshes {
                d2.draw_mesh(mesh, self.material.clone(), Matrix::identity());
            }
        });
    }

}
