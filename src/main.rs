use raylib::prelude::*;

use crate::world::generation::generate_chunk;

mod mesh_tools;
mod world;

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
