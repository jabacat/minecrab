use raylib::models::{Mesh, RaylibMesh};

use crate::render::mesh_tools::VecMesh;

// How many frames until day/night events happen.
const FRAMES_UNTIL_SUNSET: i32 = 1000;
const FRAMES_UNTIL_DUSK: i32 = 1150;
const FRAMES_UNTIL_DAWN: i32 = 1850;
const FRAMES_UNTIL_DAY: i32 = 2000;

// Computes how much "day" there is (from 1.0 during the day to 0.0 at night,
// with intermediate amounts at dawn and dusk) given the number of frames that
// have elapsed since world creation.
pub fn day_amount(frames: i32) -> f32 {
    let time_in_day = frames % FRAMES_UNTIL_DAY;
    if time_in_day < FRAMES_UNTIL_SUNSET {
        return 1.0;
    } else if time_in_day < FRAMES_UNTIL_DUSK {
        return (FRAMES_UNTIL_DUSK - time_in_day) as f32 / (FRAMES_UNTIL_DUSK - FRAMES_UNTIL_SUNSET) as f32;
    } else if time_in_day < FRAMES_UNTIL_DAWN {
        return 0.0;
    } else {
        return (time_in_day - FRAMES_UNTIL_DAWN) as f32 / (FRAMES_UNTIL_DAY - FRAMES_UNTIL_DAWN) as f32;
    }
}

// We need to render the skybox as a cube in a fixed location relative to the
// player, but NOT in a fixed orientation (like the end portal is in Minecraft,)
// for example. This needs to be far enough away that the skybox won't eclipse
// the actual terrain.
const SK_DIST: f32 = 200.;

pub fn create_skybox_mesh() -> Mesh {
    let mut vmesh = VecMesh::new();
    vmesh.vertices.extend_from_slice(&[
        // East face
         SK_DIST,  SK_DIST,  SK_DIST,
         SK_DIST,  SK_DIST, -SK_DIST,
         SK_DIST, -SK_DIST,  SK_DIST,
         SK_DIST, -SK_DIST, -SK_DIST,
         SK_DIST, -SK_DIST,  SK_DIST,
         SK_DIST,  SK_DIST, -SK_DIST,
        // West face
        -SK_DIST,  SK_DIST,  SK_DIST,
        -SK_DIST, -SK_DIST,  SK_DIST,
        -SK_DIST,  SK_DIST, -SK_DIST,
        -SK_DIST, -SK_DIST, -SK_DIST,
        -SK_DIST,  SK_DIST, -SK_DIST,
        -SK_DIST, -SK_DIST,  SK_DIST,
        // North face
         SK_DIST,  SK_DIST, -SK_DIST,
        -SK_DIST,  SK_DIST, -SK_DIST,
         SK_DIST, -SK_DIST, -SK_DIST,
        -SK_DIST, -SK_DIST, -SK_DIST,
         SK_DIST, -SK_DIST, -SK_DIST,
        -SK_DIST,  SK_DIST, -SK_DIST,
        // South face
         SK_DIST,  SK_DIST,  SK_DIST,
         SK_DIST, -SK_DIST,  SK_DIST,
        -SK_DIST,  SK_DIST,  SK_DIST,
        -SK_DIST, -SK_DIST,  SK_DIST,
        -SK_DIST,  SK_DIST,  SK_DIST,
         SK_DIST, -SK_DIST,  SK_DIST,
        // Top face
         SK_DIST,  SK_DIST,  SK_DIST,
        -SK_DIST,  SK_DIST,  SK_DIST,
         SK_DIST,  SK_DIST, -SK_DIST,
        -SK_DIST,  SK_DIST, -SK_DIST,
         SK_DIST,  SK_DIST, -SK_DIST,
        -SK_DIST,  SK_DIST,  SK_DIST,
        // Bottom face
         SK_DIST, -SK_DIST,  SK_DIST,
         SK_DIST, -SK_DIST, -SK_DIST,
        -SK_DIST, -SK_DIST,  SK_DIST,
        -SK_DIST, -SK_DIST, -SK_DIST,
        -SK_DIST, -SK_DIST,  SK_DIST,
         SK_DIST, -SK_DIST, -SK_DIST,
    ]);
    vmesh.normals.extend_from_slice(&[
        // East face faces westwards
        -1., 0., 0.,
        -1., 0., 0.,
        -1., 0., 0.,
        -1., 0., 0.,
        -1., 0., 0.,
        -1., 0., 0.,
        // West face faces eastwards
         1., 0., 0.,
         1., 0., 0.,
         1., 0., 0.,
         1., 0., 0.,
         1., 0., 0.,
         1., 0., 0.,
        // North face
        0., 0.,  1.,
        0., 0.,  1.,
        0., 0.,  1.,
        0., 0.,  1.,
        0., 0.,  1.,
        0., 0.,  1.,
        // South face
        0., 0., -1.,
        0., 0., -1.,
        0., 0., -1.,
        0., 0., -1.,
        0., 0., -1.,
        0., 0., -1.,
        // Top face
        0., -1., 0.,
        0., -1., 0.,
        0., -1., 0.,
        0., -1., 0.,
        0., -1., 0.,
        0., -1., 0.,
        // Bottom face
        0.,  1., 0.,
        0.,  1., 0.,
        0.,  1., 0.,
        0.,  1., 0.,
        0.,  1., 0.,
        0.,  1., 0.,
    ]);
    vmesh.texcoords.extend_from_slice(&[
        1., 1., 0., 1., 1., 0., 0., 0., 1., 0., 0., 1.,
        1., 1., 1., 0., 0., 1., 0., 0., 0., 1., 1., 0.,
        1., 1., 0., 1., 1., 0., 0., 0., 1., 0., 0., 1.,
        1., 1., 1., 0., 0., 1., 0., 0., 0., 1., 1., 0.,
        1., 1., 1., 0., 0., 1., 0., 0., 0., 1., 1., 0.,
        1., 1., 0., 1., 1., 0., 0., 0., 1., 0., 0., 1.,
    ]);
    let mut mesh = vmesh.to_mesh();
    unsafe { mesh.upload(false); }
    mesh
}
