use crate::agent::Agent;
use crate::map::{Map, TILE_EXIT, WALL_NONE};
use crate::player::Player;
use crate::raycaster;
use crate::text;

const SCALE: usize = 20; // pixeles por celda de mapa (mucho mas grande que el minimapa)
const BACKGROUND: u32 = 0x02_08_02;
const TITLE_COLOR: u32 = 0x00_FF_41;
const PLAYER_COLOR: u32 = 0xFF_D7_00;
const EXIT_COLOR: u32 = 0xFF_FF_FF;
const AGENT_COLOR: u32 = 0xFF_33_33;

fn put_pixel(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as usize, y as usize);
    if x < width && y < height {
        buffer[y * width + x] = color;
    }
}

fn fill_cell(buffer: &mut [u32], width: usize, height: usize, origin_x: i32, origin_y: i32, mx: usize, my: usize, color: u32) {
    let base_x = origin_x + (mx * SCALE) as i32;
    let base_y = origin_y + (my * SCALE) as i32;
    for dy in 0..SCALE as i32 {
        for dx in 0..SCALE as i32 {
            put_pixel(buffer, width, height, base_x + dx, base_y + dy, color);
        }
    }
}

fn draw_disc(buffer: &mut [u32], width: usize, height: usize, cx: i32, cy: i32, radius: i32, color: u32) {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                put_pixel(buffer, width, height, cx + dx, cy + dy, color);
            }
        }
    }
}

/// Vista de pantalla completa del laberinto entero (mucho mas grande que el
/// minimapa de la esquina), con la posicion de todos los Agentes activos
/// marcada en rojo. Reemplaza la vista 3D mientras esta activa (tecla `M`).
pub fn render(buffer: &mut [u32], screen_width: usize, screen_height: usize, map: &Map, player: &Player, agents: &[Agent]) {
    for pixel in buffer.iter_mut() {
        *pixel = BACKGROUND;
    }

    let map_width = map.width * SCALE;
    let map_height = map.height * SCALE;
    let origin_x = (screen_width as i32 - map_width as i32) / 2;
    let origin_y = (screen_height as i32 - map_height as i32) / 2;

    for my in 0..map.height {
        for mx in 0..map.width {
            let wall_id = map.get(mx as i32, my as i32);
            let color = if wall_id == WALL_NONE {
                continue;
            } else if wall_id == TILE_EXIT {
                EXIT_COLOR
            } else {
                raycaster::wall_color(wall_id)
            };
            fill_cell(buffer, screen_width, screen_height, origin_x, origin_y, mx, my, color);
        }
    }

    for agent in agents {
        if !agent.is_active() {
            continue; // eliminado/reapareciendo: no es una amenaza real ahora mismo
        }
        let ax = origin_x + (agent.x * SCALE as f64) as i32;
        let ay = origin_y + (agent.y * SCALE as f64) as i32;
        draw_disc(buffer, screen_width, screen_height, ax, ay, 6, AGENT_COLOR);
    }

    // Linea corta que muestra hacia donde mira el jugador, igual que en el minimapa.
    for i in 0..14 {
        let t = i as f64 * 0.2;
        let world_x = player.pos_x + player.dir_x * t;
        let world_y = player.pos_y + player.dir_y * t;
        let x = origin_x + (world_x * SCALE as f64) as i32;
        let y = origin_y + (world_y * SCALE as f64) as i32;
        put_pixel(buffer, screen_width, screen_height, x, y, PLAYER_COLOR);
    }
    let px = origin_x + (player.pos_x * SCALE as f64) as i32;
    let py = origin_y + (player.pos_y * SCALE as f64) as i32;
    draw_disc(buffer, screen_width, screen_height, px, py, 7, PLAYER_COLOR);

    let cx = screen_width as i32 / 2;
    text::draw_text_centered(buffer, screen_width, screen_height, cx, (origin_y - 34).max(8), 3, TITLE_COLOR, "MAPA");
    text::draw_text_centered(buffer, screen_width, screen_height, cx, screen_height as i32 - 26, 2, TITLE_COLOR, "M PARA CERRAR");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    #[test]
    fn render_paints_something_and_does_not_panic() {
        let map = Map::level(0);
        let player = Player::new(map.player_spawn.0, map.player_spawn.1);
        let agents: Vec<Agent> = map.agent_spawns.iter().map(|&(x, y)| Agent::new(x, y)).collect();

        let (width, height) = (800, 600);
        let mut buffer = vec![0u32; width * height];
        render(&mut buffer, width, height, &map, &player, &agents);

        assert!(buffer.iter().any(|&p| p != 0));
    }
}
