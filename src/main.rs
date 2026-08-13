use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn from_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new("Raycaster", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("no se pudo crear la ventana: {e}"));

    window.set_target_fps(60);

    let background_color = from_rgb(30, 30, 60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for pixel in buffer.iter_mut() {
            *pixel = background_color;
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
