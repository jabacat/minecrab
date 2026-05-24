use noise::{NoiseFn, SuperSimplex};
use raylib::prelude::*;

use crate::mesh_tools::VecMesh;

pub struct BlockData {
    non_void: bool
}

pub fn get_block_data(x: i64, y: i64, z: i64) -> BlockData {
    static SSN: std::sync::LazyLock<SuperSimplex> = std::sync::LazyLock::new(|| SuperSimplex::new(42));

    BlockData {
        non_void: SSN.get([(x as f64 / 16.) , (y as f64 / 16.) , (z as f64 / 16.) ]) > 0.5
    }
}


pub fn generate_chunk(rl: &mut RaylibHandle, thread: &RaylibThread, cx: i64, cy: i64, cz: i64) -> Model {
    let mut vertices: Vec<f32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();
    let mut tex_coords: Vec<f32> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for x in -16..16 {
        for y in -16..16 {
            for z in -16..16 {
                let (x, y, z) = (x + 32 * cx, y + 32 * cy, z + 32 * cz);
                if get_block_data(x, y, z).non_void {
                    for (dx, dy, dz) in [
                        (1, 0, 0),
                        (-1, 0, 0),
                        (0, 1, 0),
                        (0, -1, 0),
                        (0, 0, 1),
                        (0, 0, -1),
                    ] {
                        if get_block_data(x + dx, y + dy, z + dz).non_void
                        {
                            continue;
                        }

                        // if we've hit an air block, then we have a visible block face
                        // add the vertices accordingly
                        let (x, y, z) = (x as f32, y as f32, z as f32);
                        if dx == 1 {
                            // FIXME: how do I stop this from getting formatted
                            // #[rustfmt::skip]
                            vertices.extend_from_slice(&[
                                x + 1., y, z,
                                x + 1., y + 1., z,
                                x + 1., y + 1., z + 1.,
                                x + 1., y, z + 1.,
                            ]);
                            tex_coords.extend_from_slice(&[
                                0.1, 0.0,
                                0.1, 0.1,
                                0.0, 0.1,
                                0.0, 0.0,
                            ]);
                        } else if dx == -1 {
                            vertices.extend_from_slice(&[
                                x, y, z,
                                x, y, z + 1.,
                                x, y + 1., z + 1.,
                                x, y + 1., z,
                            ]);
                            tex_coords.extend_from_slice(&[
                                0.0, 0.0,
                                0.1, 0.0,
                                0.1, 0.1,
                                0.0, 0.1,
                            ]);
                        } else if dy == 1 {
                            vertices.extend_from_slice(&[
                                x, y + 1., z,
                                x, y + 1., z + 1.,
                                x + 1., y + 1., z + 1.,
                                x + 1., y + 1., z,
                            ]);
                            tex_coords.extend_from_slice(&[
                                0.0, 0.1,
                                0.0, 0.0,
                                0.1, 0.0,
                                0.1, 0.1,
                            ]);
                        } else if dy == -1 {
                            vertices.extend_from_slice(&[
                                x, y, z,
                                x + 1., y, z,
                                x + 1., y, z + 1.,
                                x, y, z + 1.,
                            ]);
                            tex_coords.extend_from_slice(&[
                                0.0, 0.0,
                                0.1, 0.0,
                                0.1, 0.1,
                                0.0, 0.1,
                            ]);
                        } else if dz == 1 {
                            vertices.extend_from_slice(&[
                                x, y, z + 1.0,
                                x + 1., y, z + 1.0,
                                x + 1., y + 1., z + 1.0,
                                x, y + 1., z + 1.0,
                            ]);
                            tex_coords.extend_from_slice(&[
                                0.0, 0.0,
                                0.1, 0.0,
                                0.1, 0.1,
                                0.0, 0.1,
                            ]);
                        } else if dz == -1 {
                            vertices.extend_from_slice(&[
                                x, y, z,
                                x, y + 1., z,
                                x + 1., y + 1., z,
                                x + 1., y, z,
                            ]);
                            tex_coords.extend_from_slice(&[
                                0.1, 0.0,
                                0.1, 0.1,
                                0.0, 0.1,
                                0.0, 0.0,
                            ]);
                        }
                        // dx, dy, dz give us the normals for this face
                        let (dx, dy, dz) = (dx as f32, dy as f32, dz as f32);
                        normals.extend_from_slice(&[
                            dx, dy, dz,
                            dx, dy, dz,
                            dx, dy, dz,
                            dx, dy, dz,
                        ]);
                    }
                }
            }
        }
    }

    dbg!(vertices.len());
    dbg!(vertices.len() % 12);
    dbg!(normals.len());
    dbg!(tex_coords.len());
    dbg!(normals.len());
    dbg!(normals.len() % 12);

    indices.resize(vertices.len() / 2, 0);
    for i in 0..vertices.len() / 12 {
        // FIXME: these type casts are really ugly; there must be a better way
        // nvm I think this shadowing solution is pretty good
        let k = i as u16;
        indices[6 * i] = 4 * k;
        indices[6 * i + 1] = 4 * k + 1;
        indices[6 * i + 2] = 4 * k + 2;
        indices[6 * i + 3] = 4 * k;
        indices[6 * i + 4] = 4 * k + 2;
        indices[6 * i + 5] = 4 * k + 3;
    }

    dbg!(indices.len());
    dbg!(indices.len() % 6);

    let mut vmesh = VecMesh::new();
    vmesh.vertices = vertices;
    vmesh.normals = normals;
    vmesh.texcoords = tex_coords;
    vmesh.indices = indices;

    let mut mesh = vmesh.to_mesh();
    unsafe { mesh.upload(false) };

    // FIXME: my theory is that vao and vbo should now be initialized
    // unfortunately there seems to be no way to check (?)
    // dbg!(mesh.to_raw().vaoId);
    // dbg!(mesh.to_raw().vboId);

    let model = rl.load_model_from_mesh(thread, unsafe { mesh.make_weak() }).unwrap();

    model
}
