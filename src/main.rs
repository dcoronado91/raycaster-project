mod agent;
mod map;
mod minimap;
mod player;
mod raycaster;
mod sprite;

use agent::Agent;
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
    let spawn_x = player.pos_x;
    let spawn_y = player.pos_y;
    let mut agent_smith = Agent::new(21.0, 21.0);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut z_buffer: Vec<f64> = vec![0.0; WIDTH];

    let mut window = Window::new("Raycaster", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("no se pudo crear la ventana: {e}"));

    window.set_target_fps(60);
    window.set_cursor_visibility(false);

    let ceiling_color = from_rgb(4, 10, 4);
    let floor_color = from_rgb(8, 16, 8);

    let start_time = Instant::now();
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

        agent_smith.update(&map, &player, dt);
        if agent_smith.is_touching_player(&player) {
            // Reinicio temporal al detectar contacto; la pantalla de Game Over
            // llega cuando conectemos la maquina de estados (proximo commit).
            eprintln!("Un Agente te atrapo. Reiniciando posicion...");
            player.pos_x = spawn_x;
            player.pos_y = spawn_y;
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

        raycaster::render(&mut buffer, &mut z_buffer, WIDTH, HEIGHT, &map, &player);

        let elapsed = start_time.elapsed().as_secs_f64();
        let walk_phase = (elapsed * 4.0) as i32;
        sprite::render(
            &mut buffer,
            &z_buffer,
            WIDTH,
            HEIGHT,
            &player,
            agent_smith.x,
            agent_smith.y,
            |tx, ty| agent::agent_pixel(tx, ty, walk_phase),
        );

        minimap::render(&mut buffer, WIDTH, HEIGHT, &map, &player);

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
