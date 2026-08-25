mod agent;
mod map;
mod minimap;
mod platform;
mod player;
mod raycaster;
mod sprite;
mod weapon;

use agent::Agent;
use map::Map;
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use player::Player;
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MOVE_SPEED: f64 = 3.0; // celdas de mapa por segundo
const MOUSE_SENSITIVITY: f64 = 0.0025; // radianes por pixel de movimiento del mouse
const SHOOT_RANGE: f64 = 12.0; // celdas de mapa
const SHOOT_COOLDOWN: f64 = 0.4; // segundos entre disparos
const MUZZLE_FLASH_DURATION: f64 = 0.12; // segundos que dura el destello visual

fn from_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Color de un texel (tx, ty) del faro que marca la salida: un resplandor
/// circular blanco-verdoso, mas brillante al centro, que "respira" con
/// `pulse` (-1.0 a 1.0) para que se note incluso desde lejos.
fn exit_beacon_pixel(tx: usize, ty: usize, pulse: f64) -> Option<u32> {
    let dx = tx as f64 - 32.0;
    let dy = ty as f64 - 32.0;
    let dist = (dx * dx + dy * dy).sqrt();
    let radius = 20.0 + pulse * 6.0;
    if dist > radius {
        return None;
    }
    let brightness = (140.0 + (1.0 - dist / radius) * 115.0) as u32;
    Some((brightness << 16) | (0xFF << 8) | brightness)
}

fn main() {
    let map = Map::level(0);
    let mut player = Player::new(map.player_spawn.0, map.player_spawn.1);
    let spawn_x = map.player_spawn.0;
    let spawn_y = map.player_spawn.1;
    // Varios Agentes a la vez (hasta 6 en el nivel mas grande), repartidos en
    // distintos puntos del laberinto. Cada uno reaparece en su propio origen.
    let mut agents: Vec<Agent> = map
        .agent_spawns
        .iter()
        .map(|&(x, y)| Agent::new(x, y))
        .collect();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut z_buffer: Vec<f64> = vec![0.0; WIDTH];

    let mut window = Window::new("Raycaster", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("no se pudo crear la ventana: {e}"));

    window.set_target_fps(60);
    window.set_cursor_visibility(false);
    platform::confine_cursor(window.get_window_handle());

    let ceiling_color = from_rgb(4, 10, 4);
    let floor_color = from_rgb(8, 16, 8);

    let start_time = Instant::now();
    let mut last_frame = Instant::now();
    let mut mouse_was_down = false;
    let mut shoot_cooldown = 0.0f64;
    let mut muzzle_flash_timer = 0.0f64;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Windows libera el confinamiento del cursor si la ventana pierde el
        // foco, asi que se vuelve a aplicar en cada cuadro (llamada barata).
        platform::confine_cursor(window.get_window_handle());

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f64();
        last_frame = now;

        shoot_cooldown = (shoot_cooldown - dt).max(0.0);
        muzzle_flash_timer = (muzzle_flash_timer - dt).max(0.0);

        let mouse_down = window.get_mouse_down(MouseButton::Left);
        let shoot_pressed = mouse_down && !mouse_was_down && shoot_cooldown <= 0.0;
        mouse_was_down = mouse_down;
        if shoot_pressed {
            shoot_cooldown = SHOOT_COOLDOWN;
            muzzle_flash_timer = MUZZLE_FLASH_DURATION;
        }

        // El cursor se recentra cada cuadro (mas abajo), asi que la rotacion
        // se calcula contra el centro de la ventana en vez de contra la
        // posicion del cuadro anterior: eso permite girar sin limites.
        if let Some((mouse_x, _)) = window.get_mouse_pos(MouseMode::Pass) {
            let delta_x = mouse_x as f64 - WIDTH as f64 / 2.0;
            if delta_x != 0.0 {
                player.rotate(delta_x * MOUSE_SENSITIVITY);
            }
        }
        platform::recenter_cursor(window.get_window_handle(), WIDTH as i32, HEIGHT as i32);

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

        for agent in agents.iter_mut() {
            agent.update(&map, &player, dt);
        }
        if agents.iter().any(|agent| agent.is_touching_player(&player)) {
            // Reinicio temporal al detectar contacto; la pantalla de Game Over
            // llega cuando conectemos la maquina de estados (proximo commit).
            eprintln!("Un Agente te atrapo. Reiniciando posicion...");
            player.pos_x = spawn_x;
            player.pos_y = spawn_y;
        }
        if map.is_exit(player.pos_x, player.pos_y) {
            // Reinicio temporal al llegar a la salida; la pantalla de exito
            // llega cuando conectemos la maquina de estados (proximo commit).
            eprintln!("Escapaste del laberinto. Reiniciando posicion...");
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

        if shoot_pressed {
            // Si varios Agentes caen dentro de la mira, se le acierta al mas cercano.
            let mut closest: Option<(usize, f64)> = None;
            for (i, agent) in agents.iter().enumerate() {
                if !agent.is_active()
                    || !sprite::is_targetable(&player, agent.x, agent.y, &z_buffer, WIDTH, SHOOT_RANGE)
                {
                    continue;
                }
                let dx = agent.x - player.pos_x;
                let dy = agent.y - player.pos_y;
                let dist_sq = dx * dx + dy * dy;
                if closest.is_none_or(|(_, best)| dist_sq < best) {
                    closest = Some((i, dist_sq));
                }
            }
            if let Some((i, _)) = closest {
                agents[i].hit();
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let walk_phase = (elapsed * 4.0) as i32;

        // Se pintan de mas lejos a mas cerca para que un agente al frente
        // tape correctamente a uno detras (pintor's algorithm entre sprites).
        let mut draw_order: Vec<usize> = (0..agents.len()).collect();
        draw_order.sort_by(|&a, &b| {
            let dist_sq = |agent: &Agent| {
                let dx = agent.x - player.pos_x;
                let dy = agent.y - player.pos_y;
                dx * dx + dy * dy
            };
            dist_sq(&agents[b]).total_cmp(&dist_sq(&agents[a]))
        });
        for i in draw_order {
            if agents[i].is_active() {
                sprite::render(
                    &mut buffer,
                    &z_buffer,
                    WIDTH,
                    HEIGHT,
                    &player,
                    agents[i].x,
                    agents[i].y,
                    |tx, ty| agent::agent_pixel(tx, ty, walk_phase),
                );
            }
        }

        let pulse = (elapsed * 3.0).sin();
        sprite::render(
            &mut buffer,
            &z_buffer,
            WIDTH,
            HEIGHT,
            &player,
            map.exit.0,
            map.exit.1,
            |tx, ty| exit_beacon_pixel(tx, ty, pulse),
        );

        minimap::render(&mut buffer, WIDTH, HEIGHT, &map, &player);

        let is_moving = move_x != 0.0 || move_y != 0.0;
        let bob_amplitude = if is_moving { 6.0 } else { 2.0 };
        let bob_offset = (elapsed * 8.0).sin() * bob_amplitude;
        weapon::render(
            &mut buffer,
            WIDTH,
            HEIGHT,
            muzzle_flash_timer / MUZZLE_FLASH_DURATION,
            bob_offset,
        );

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    platform::release_cursor();
}
