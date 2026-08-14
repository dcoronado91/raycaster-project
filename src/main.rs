mod map;
mod minimap;
mod player;
mod raycaster;

use map::Map;
use minifb::{Key, MouseMode, Window, WindowOptions};
use player::Player;
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MOVE_SPEED: f64 = 3.0; // celdas de mapa por segundo
const MOUSE_SENSITIVITY: f64 = 0.0025; // radianes por pixel de movimiento del mouse

fn from_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn main() {
    let map = Map::level_1();
    let mut player = Player::new(12.0, 12.0);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new("Raycaster", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("no se pudo crear la ventana: {e}"));

    window.set_target_fps(60);
    window.set_cursor_visibility(false);

    let ceiling_color = from_rgb(40, 40, 40);
    let floor_color = from_rgb(70, 70, 70);

    let mut last_frame = Instant::now();
    let mut last_mouse_x = window
        .get_mouse_pos(MouseMode::Pass)
        .map(|(x, _)| x)
        .unwrap_or(0.0);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f64();
        last_frame = now;

        // minifb no permite recentrar el cursor, asi que la rotacion se basa
        // en el delta de posicion frame a frame en vez de un mouse-look "infinito".
        if let Some((mouse_x, _)) = window.get_mouse_pos(MouseMode::Pass) {
            let delta_x = (mouse_x - last_mouse_x) as f64;
            last_mouse_x = mouse_x;
            if delta_x != 0.0 {
                player.rotate(delta_x * MOUSE_SENSITIVITY);
            }
        }

        let mut move_x = 0.0;
        let mut move_y = 0.0;
        if window.is_key_down(Key::W) {
            move_x += player.dir_x;
            move_y += player.dir_y;
        }
        if window.is_key_down(Key::S) {
            move_x -= player.dir_x;
            move_y -= player.dir_y;
        }
        if window.is_key_down(Key::A) {
            move_x -= -player.dir_y;
            move_y -= player.dir_x;
        }
        if window.is_key_down(Key::D) {
            move_x += -player.dir_y;
            move_y += player.dir_x;
        }
        if move_x != 0.0 || move_y != 0.0 {
            let len = (move_x * move_x + move_y * move_y).sqrt();
            let step = MOVE_SPEED * dt;
            player.try_move(&map, move_x / len * step, move_y / len * step);
        }

        for y in 0..HEIGHT {
            let row_color = if y < HEIGHT / 2 {
                ceiling_color
            } else {
                floor_color
            };
            for x in 0..WIDTH {
                buffer[y * WIDTH + x] = row_color;
            }
        }

        raycaster::render(&mut buffer, WIDTH, HEIGHT, &map, &player);
        minimap::render(&mut buffer, WIDTH, HEIGHT, &map, &player);

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
