use noise::{NoiseFn, SuperSimplex};
use raylib::prelude::*;

use crate::mesh_tools::VecMesh;

mod mesh_tools;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Hello, world!")
        .vsync()
        .highdpi()
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(3.0, 3.0, 3.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    let mut first_click = false;

    let texture = unsafe {
        let mut t = rl.load_texture(&thread, "assets/full-textures.png").unwrap();
        t.gen_texture_mipmaps();
        t.unwrap()
    };

    let mut model = generate_chunk(&mut rl, &thread, 0, 0, 0);
    
    let materials = model.materials_mut();
    let material = &mut materials[0];
    let maps = material.maps_mut();
    maps[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].texture = texture;

    while !rl.window_should_close() {
        // require a click on the window before updating camera so the camera
        // doesn't fly away when the cursor enters the window at first
        if !first_click {
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                first_click = true;
                rl.disable_cursor();
            }
        } else {
            rl.update_camera(&mut camera, CameraMode::CAMERA_FIRST_PERSON);
        }

        rl.draw(&thread, |mut d| {
            d.clear_background(Color::LIGHTBLUE);

            d.draw_mode3D(camera, |mut d2, _camera| {
                d2.draw_model(&model, Vector3::zero(), 1.0, Color::WHITE);
            });

            if !first_click {
                d.draw_text(
                    "WIP: Click to start updating camera",
                    20,
                    20,
                    16,
                    Color::DARKGREEN,
                );
            }
        });
    }
}


// FIXME: use x y z offsets properly
fn generate_chunk(rl: &mut RaylibHandle, thread: &RaylibThread, x: i64, y: i64, z: i64) -> Model {
    let ssn = SuperSimplex::new(42);

    let mut vertices: Vec<f32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();
    let mut tex_coords: Vec<f32> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for x in -16..16 {
        for y in -16..16 {
            for z in -16..16 {
                if ssn.get([(x as f64 / 16.) , (y as f64 / 16.) , (z as f64 / 16.) ]) > 0.5 {
                    for (dx, dy, dz) in [
                        (1, 0, 0),
                        (-1, 0, 0),
                        (0, 1, 0),
                        (0, -1, 0),
                        (0, 0, 1),
                        (0, 0, -1),
                    ] {
                        if ssn.get([
                            ((x + dx) as f64 / 16.),
                            ((y + dy) as f64 / 16.),
                            ((z + dz) as f64 / 16.),
                        ]) > 0.5
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
