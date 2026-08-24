use crate::map::{Map, WALL_NONE};
use crate::player::Player;
use crate::raycaster;

const SCALE: usize = 6; // pixeles por celda de mapa
const MARGIN: usize = 10;
const BACKGROUND: u32 = 0x02_08_02;
const PLAYER_COLOR: u32 = 0xFF_D7_00;

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

/// Dibuja el minimapa en la esquina superior derecha de la pantalla, encima
/// de la escena 3D (no al lado del mapa principal).
pub fn render(buffer: &mut [u32], screen_width: usize, screen_height: usize, map: &Map, player: &Player) {
    let minimap_width = map.width * SCALE;
    let minimap_height = map.height * SCALE;

    let origin_x = (screen_width - minimap_width - MARGIN) as i32;
    let origin_y = MARGIN as i32;

    for y in 0..minimap_height as i32 {
        for x in 0..minimap_width as i32 {
            put_pixel(buffer, screen_width, screen_height, origin_x + x, origin_y + y, BACKGROUND);
        }
    }

    for my in 0..map.height {
        for mx in 0..map.width {
            let wall_id = map.get(mx as i32, my as i32);
            if wall_id == WALL_NONE {
                continue;
            }
            let color = raycaster::wall_color(wall_id);
            fill_cell(buffer, screen_width, screen_height, origin_x, origin_y, mx, my, color);
        }
    }

    let player_x = origin_x + (player.pos_x * SCALE as f64) as i32;
    let player_y = origin_y + (player.pos_y * SCALE as f64) as i32;

    // Linea corta que muestra hacia donde mira el jugador.
    for i in 0..10 {
        let t = i as f64 * 0.25;
        let world_x = player.pos_x + player.dir_x * t;
        let world_y = player.pos_y + player.dir_y * t;
        let px = origin_x + (world_x * SCALE as f64) as i32;
        let py = origin_y + (world_y * SCALE as f64) as i32;
        put_pixel(buffer, screen_width, screen_height, px, py, PLAYER_COLOR);
    }

    draw_disc(buffer, screen_width, screen_height, player_x, player_y, 3, PLAYER_COLOR);
}
