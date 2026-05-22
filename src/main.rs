use raylib::prelude::*;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Hello, world!")
        // FIXME: flickering
        //.vsync()
        .build();

    rl.disable_cursor();

    let mut camera = Camera3D::perspective(
        Vector3::new(3.0, 3.0, 3.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

    while !rl.window_should_close() {
        rl.update_camera(&mut camera, CameraMode::CAMERA_FIRST_PERSON);
        
        rl.draw(&thread, |mut d| {
            d.clear_background(Color::LIGHTBLUE);

            d.draw_mode3D(camera, |mut d2, _camera| {
                d2.draw_cube(Vector3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0, Color::WHITE);
            });
        });
    }
}
