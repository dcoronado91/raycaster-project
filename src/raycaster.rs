use crate::map::{Map, WALL_BRICK, WALL_MOSS, WALL_STONE, WALL_WOOD};
use crate::player::Player;

const MAX_SHADE_DISTANCE: f64 = 16.0;

pub fn wall_color(wall_id: u8) -> u32 {
    match wall_id {
        WALL_BRICK => 0xB2_22_22,
        WALL_STONE => 0x80_80_80,
        WALL_WOOD => 0x8B_5A_2B,
        WALL_MOSS => 0x2E_8B_57,
        _ => 0xFF_00_FF, // magenta: id de pared desconocido, facil de detectar
    }
}

/// Oscurece un color RGB multiplicando cada canal por `factor` (0.0 a 1.0).
fn shade_color(color: u32, factor: f64) -> u32 {
    let r = ((color >> 16) & 0xFF) as f64;
    let g = ((color >> 8) & 0xFF) as f64;
    let b = (color & 0xFF) as f64;

    let r = (r * factor).clamp(0.0, 255.0) as u32;
    let g = (g * factor).clamp(0.0, 255.0) as u32;
    let b = (b * factor).clamp(0.0, 255.0) as u32;

    (r << 16) | (g << 8) | b
}

/// Pinta las paredes visibles usando DDA (Digital Differential Analysis):
/// para cada columna de pantalla se lanza un rayo desde el jugador y se
/// avanza casilla por casilla hasta chocar con una pared.
pub fn render(buffer: &mut [u32], width: usize, height: usize, map: &Map, player: &Player) {
    for x in 0..width {
        let camera_x = 2.0 * x as f64 / width as f64 - 1.0;
        let ray_dir_x = player.dir_x + player.plane_x * camera_x;
        let ray_dir_y = player.dir_y + player.plane_y * camera_x;

        let mut map_x = player.pos_x as i32;
        let mut map_y = player.pos_y as i32;

        let delta_dist_x = if ray_dir_x == 0.0 {
            f64::INFINITY
        } else {
            (1.0 / ray_dir_x).abs()
        };
        let delta_dist_y = if ray_dir_y == 0.0 {
            f64::INFINITY
        } else {
            (1.0 / ray_dir_y).abs()
        };

        let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
            (-1, (player.pos_x - map_x as f64) * delta_dist_x)
        } else {
            (1, (map_x as f64 + 1.0 - player.pos_x) * delta_dist_x)
        };
        let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
            (-1, (player.pos_y - map_y as f64) * delta_dist_y)
        } else {
            (1, (map_y as f64 + 1.0 - player.pos_y) * delta_dist_y)
        };

        // side: 0 = se choco con una pared "vertical" (variando x), 1 = "horizontal" (variando y)
        let mut side: u8;
        loop {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                side = 0;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                side = 1;
            }
            if map.is_wall(map_x, map_y) {
                break;
            }
        }
        let wall_id = map.get(map_x, map_y);

        let perp_wall_dist = if side == 0 {
            side_dist_x - delta_dist_x
        } else {
            side_dist_y - delta_dist_y
        };

        let line_height = (height as f64 / perp_wall_dist) as i32;
        let draw_start = (-line_height / 2 + height as i32 / 2).max(0);
        let draw_end = (line_height / 2 + height as i32 / 2).min(height as i32 - 1);

        // Caras "horizontales" (side 1) un poco mas oscuras que las "verticales" (side 0)
        // para que se note el contorno de las paredes; ademas se atenua con la distancia.
        let side_factor = if side == 1 { 0.7 } else { 1.0 };
        let distance_factor = (1.0 - (perp_wall_dist / MAX_SHADE_DISTANCE).min(1.0) * 0.8).max(0.2);
        let color = shade_color(wall_color(wall_id), side_factor * distance_factor);

        for y in draw_start..=draw_end {
            buffer[y as usize * width + x] = color;
        }
    }
}
