mod agent;
mod hud;
mod map;
mod minimap;
mod platform;
mod player;
mod raycaster;
mod rng;
mod screens;
mod sprite;
mod text;
mod weapon;

use agent::Agent;
use map::Map;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use player::Player;
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MOVE_SPEED: f64 = 3.0; // celdas de mapa por segundo
const MOUSE_SENSITIVITY: f64 = 0.0025; // radianes por pixel de movimiento del mouse
const SHOOT_RANGE: f64 = 12.0; // celdas de mapa
const SHOOT_COOLDOWN: f64 = 0.4; // segundos entre disparos
const MUZZLE_FLASH_DURATION: f64 = 0.12; // segundos que dura el destello visual
const MAX_AMMO: u32 = 6;
const RELOAD_DURATION: f64 = 1.2; // segundos que tarda una recarga

/// En que pantalla esta el juego. `Playing` es la unica que corre el
/// raycaster; las demas son menus de texto sobre el fondo animado.
enum GameState {
    Welcome,
    Playing,
    GameOver,
    Success,
}

/// Todo lo que cambia al (re)iniciar un nivel: el laberinto, el jugador y
/// los Agentes que lo persiguen. Reiniciar un nivel (reintentar tras un
/// Game Over, o pasar al siguiente) simplemente crea una Session nueva.
struct Session {
    map: Map,
    player: Player,
    agents: Vec<Agent>,
    ammo: u32,
    reload_timer: f64, // > 0.0 mientras se esta recargando
}

impl Session {
    fn start(level_index: usize) -> Self {
        let map = Map::level(level_index);
        let player = Player::new(map.player_spawn.0, map.player_spawn.1);
        let agents = map.agent_spawns.iter().map(|&(x, y)| Agent::new(x, y)).collect();
        Session {
            map,
            player,
            agents,
            ammo: MAX_AMMO,
            reload_timer: 0.0,
        }
    }
}

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

    let mut state = GameState::Welcome;
    let mut selected_level: usize = 0;
    let mut session = Session::start(0);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Windows libera el confinamiento del cursor si la ventana pierde el
        // foco, asi que se vuelve a aplicar en cada cuadro (llamada barata).
        platform::confine_cursor(window.get_window_handle());

        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f64();
        last_frame = now;
        let elapsed = start_time.elapsed().as_secs_f64();

        shoot_cooldown = (shoot_cooldown - dt).max(0.0);
        muzzle_flash_timer = (muzzle_flash_timer - dt).max(0.0);

        let mouse_down = window.get_mouse_down(MouseButton::Left);
        let want_shoot = mouse_down && !mouse_was_down && shoot_cooldown <= 0.0;
        mouse_was_down = mouse_down;

        let mut shoot_pressed = false;
        if want_shoot {
            shoot_cooldown = SHOOT_COOLDOWN;
            if session.reload_timer <= 0.0 && session.ammo > 0 {
                session.ammo -= 1;
                muzzle_flash_timer = MUZZLE_FLASH_DURATION;
                shoot_pressed = true;
            } else if session.reload_timer <= 0.0 {
                // Pistola vacia: un clic en vacio dispara la recarga solo.
                session.reload_timer = RELOAD_DURATION;
            }
        }

        match state {
            GameState::Welcome => {
                let level_count = Map::level_count();
                if window.is_key_pressed(Key::Down, KeyRepeat::No) || window.is_key_pressed(Key::S, KeyRepeat::No) {
                    selected_level = (selected_level + 1) % level_count;
                }
                if window.is_key_pressed(Key::Up, KeyRepeat::No) || window.is_key_pressed(Key::W, KeyRepeat::No) {
                    selected_level = (selected_level + level_count - 1) % level_count;
                }
                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    session = Session::start(selected_level);
                    state = GameState::Playing;
                }
                screens::draw_welcome(&mut buffer, WIDTH, HEIGHT, elapsed, selected_level, level_count);
            }

            GameState::Playing => {
                // El cursor se recentra cada cuadro (mas abajo), asi que la
                // rotacion se calcula contra el centro de la ventana en vez
                // de contra la posicion del cuadro anterior: eso permite
                // girar sin limites.
                if let Some((mouse_x, _)) = window.get_mouse_pos(MouseMode::Pass) {
                    let delta_x = mouse_x as f64 - WIDTH as f64 / 2.0;
                    if delta_x != 0.0 {
                        session.player.rotate(delta_x * MOUSE_SENSITIVITY);
                    }
                }
                platform::recenter_cursor(window.get_window_handle(), WIDTH as i32, HEIGHT as i32);

                if session.reload_timer > 0.0 {
                    session.reload_timer = (session.reload_timer - dt).max(0.0);
                    if session.reload_timer <= 0.0 {
                        session.ammo = MAX_AMMO;
                    }
                }
                if window.is_key_pressed(Key::R, KeyRepeat::No) && session.reload_timer <= 0.0 && session.ammo < MAX_AMMO {
                    session.reload_timer = RELOAD_DURATION;
                }

                let mut move_x = 0.0;
                let mut move_y = 0.0;
                if window.is_key_down(Key::W) {
                    move_x += session.player.dir_x;
                    move_y += session.player.dir_y;
                }
                if window.is_key_down(Key::S) {
                    move_x -= session.player.dir_x;
                    move_y -= session.player.dir_y;
                }
                if window.is_key_down(Key::A) {
                    move_x -= -session.player.dir_y;
                    move_y -= session.player.dir_x;
                }
                if window.is_key_down(Key::D) {
                    move_x += -session.player.dir_y;
                    move_y += session.player.dir_x;
                }
                if move_x != 0.0 || move_y != 0.0 {
                    let len = (move_x * move_x + move_y * move_y).sqrt();
                    let step = MOVE_SPEED * dt;
                    session.player.try_move(&session.map, move_x / len * step, move_y / len * step);
                }

                for agent in session.agents.iter_mut() {
                    agent.update(&session.map, &session.player, dt);
                }

                for y in 0..HEIGHT {
                    let row_color = if y < HEIGHT / 2 { ceiling_color } else { floor_color };
                    for x in 0..WIDTH {
                        buffer[y * WIDTH + x] = row_color;
                    }
                }

                raycaster::render(&mut buffer, &mut z_buffer, WIDTH, HEIGHT, &session.map, &session.player);

                if shoot_pressed {
                    // Si varios Agentes caen dentro de la mira, se le acierta al mas cercano.
                    let mut closest: Option<(usize, f64)> = None;
                    for (i, agent) in session.agents.iter().enumerate() {
                        if !agent.is_active()
                            || !sprite::is_targetable(&session.player, agent.x, agent.y, &z_buffer, WIDTH, SHOOT_RANGE)
                        {
                            continue;
                        }
                        let dx = agent.x - session.player.pos_x;
                        let dy = agent.y - session.player.pos_y;
                        let dist_sq = dx * dx + dy * dy;
                        if closest.is_none_or(|(_, best)| dist_sq < best) {
                            closest = Some((i, dist_sq));
                        }
                    }
                    if let Some((i, _)) = closest {
                        session.agents[i].hit();
                    }
                }

                let walk_phase = (elapsed * 4.0) as i32;

                // Se pintan de mas lejos a mas cerca para que un agente al
                // frente tape correctamente a uno detras (pintor's algorithm).
                let mut draw_order: Vec<usize> = (0..session.agents.len()).collect();
                draw_order.sort_by(|&a, &b| {
                    let dist_sq = |agent: &Agent| {
                        let dx = agent.x - session.player.pos_x;
                        let dy = agent.y - session.player.pos_y;
                        dx * dx + dy * dy
                    };
                    dist_sq(&session.agents[b]).total_cmp(&dist_sq(&session.agents[a]))
                });
                for i in draw_order {
                    if session.agents[i].is_active() {
                        sprite::render(
                            &mut buffer,
                            &z_buffer,
                            WIDTH,
                            HEIGHT,
                            &session.player,
                            session.agents[i].x,
                            session.agents[i].y,
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
                    &session.player,
                    session.map.exit.0,
                    session.map.exit.1,
                    |tx, ty| exit_beacon_pixel(tx, ty, pulse),
                );

                minimap::render(&mut buffer, WIDTH, HEIGHT, &session.map, &session.player);

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
                hud::draw_ammo(&mut buffer, WIDTH, HEIGHT, session.ammo, MAX_AMMO, session.reload_timer > 0.0);

                if session.agents.iter().any(|agent| agent.is_touching_player(&session.player, &session.map)) {
                    state = GameState::GameOver;
                } else if session.map.is_exit(session.player.pos_x, session.player.pos_y) {
                    state = GameState::Success;
                }
            }

            GameState::GameOver => {
                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    session = Session::start(selected_level);
                    state = GameState::Playing;
                }
                screens::draw_game_over(&mut buffer, WIDTH, HEIGHT, elapsed);
            }

            GameState::Success => {
                let is_last_level = selected_level + 1 >= Map::level_count();
                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    if is_last_level {
                        selected_level = 0;
                        state = GameState::Welcome;
                    } else {
                        selected_level += 1;
                        session = Session::start(selected_level);
                        state = GameState::Playing;
                    }
                }
                screens::draw_success(&mut buffer, WIDTH, HEIGHT, elapsed, is_last_level);
            }
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    platform::release_cursor();
}
