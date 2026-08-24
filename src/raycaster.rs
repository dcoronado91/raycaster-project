use crate::map::{Map, WALL_CIRCUIT, WALL_CODE, WALL_CONCRETE, WALL_SERVER};
use crate::player::Player;

// Distancia mas corta que antes: la visibilidad se apaga rapido para dar
// sensacion de tunel oscuro/opresivo, acorde al tono "de miedo" del laberinto.
const MAX_SHADE_DISTANCE: f64 = 10.0;

/// Paleta verde "digital", inspirada en Matrix: nada de colores calidos,
/// todo en tonos de verde sobre negro para que el mundo se sienta artificial.
pub fn wall_color(wall_id: u8) -> u32 {
    match wall_id {
        WALL_CONCRETE => 0x1A_2A_1A,
        WALL_SERVER => 0x0D_3B_1E,
        WALL_CIRCUIT => 0x14_52_14,
        WALL_CODE => 0x00_FF_41,
        _ => 0xFF_00_FF, // magenta: id de pared desconocido, facil de detectar
    }
}

/// Oscurece (o aclara) un color RGB multiplicando cada canal por `factor`.
/// pub(crate) porque el modulo de sprites tambien lo usa para sombrear por distancia.
pub(crate) fn shade_color(color: u32, factor: f64) -> u32 {
    let r = ((color >> 16) & 0xFF) as f64;
    let g = ((color >> 8) & 0xFF) as f64;
    let b = (color & 0xFF) as f64;

    let r = (r * factor).clamp(0.0, 255.0) as u32;
    let g = (g * factor).clamp(0.0, 255.0) as u32;
    let b = (b * factor).clamp(0.0, 255.0) as u32;

    (r << 16) | (g << 8) | b
}

/// Resolucion (en texeles) de las texturas proceduales.
const TEX_SIZE: usize = 64;

/// Color de un texel (tx, ty) de la textura de `wall_id`. En vez de cargar
/// imagenes, cada tipo de pared se genera con un patron matematico simple
/// aplicado como un factor de brillo sobre su color base.
fn texture_pixel(wall_id: u8, tx: usize, ty: usize) -> u32 {
    let pattern_factor = match wall_id {
        WALL_CONCRETE => concrete_pattern(tx, ty),
        WALL_SERVER => server_pattern(tx, ty),
        WALL_CIRCUIT => circuit_pattern(tx, ty),
        WALL_CODE => code_pattern(tx, ty),
        _ => 1.0,
    };
    shade_color(wall_color(wall_id), pattern_factor)
}

/// Bloques de concreto con juntas marcadas, como un pasillo de instalaciones.
fn concrete_pattern(tx: usize, ty: usize) -> f64 {
    const BLOCK_W: usize = 16;
    const BLOCK_H: usize = 8;
    let row_offset = if (ty / BLOCK_H) % 2 == 0 { 0 } else { BLOCK_W / 2 };
    let is_joint = ty % BLOCK_H == 0 || (tx + row_offset) % BLOCK_W == 0;
    if is_joint { 0.5 } else { 1.0 }
}

/// Rejilla tipo rack de servidores, alternando paneles claros/oscuros.
fn server_pattern(tx: usize, ty: usize) -> f64 {
    const BLOCK: usize = 8;
    if ((tx / BLOCK) + (ty / BLOCK)) % 2 == 0 { 0.75 } else { 1.15 }
}

/// Paneles verticales con conductos/cables (costuras oscuras entre paneles).
fn circuit_pattern(tx: usize, ty: usize) -> f64 {
    const PANEL: usize = 8;
    if tx % PANEL == 0 {
        0.5
    } else {
        1.0 - ((ty * 37 + tx * 17) % 13) as f64 * 0.02
    }
}

/// "Codigo" verde brillante moteado, como estatica digital sobre el panel.
fn code_pattern(tx: usize, ty: usize) -> f64 {
    // Hash barato para lograr un moteado pseudoaleatorio sin depender de assets.
    let h = (tx.wrapping_mul(374_761_393) ^ ty.wrapping_mul(668_265_263)) & 0xFF;
    0.55 + (h as f64 / 255.0) * 0.7
}

/// Pinta las paredes visibles usando DDA (Digital Differential Analysis):
/// para cada columna de pantalla se lanza un rayo desde el jugador y se
/// avanza casilla por casilla hasta chocar con una pared. De paso llena
/// `z_buffer` con la distancia de pared por columna, para que el modulo de
/// sprites sepa cuando un sprite queda oculto detras de una pared.
pub fn render(buffer: &mut [u32], z_buffer: &mut [f64], width: usize, height: usize, map: &Map, player: &Player) {
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
        z_buffer[x] = perp_wall_dist;

        let line_height = (height as f64 / perp_wall_dist) as i32;
        let draw_start = (-line_height / 2 + height as i32 / 2).max(0);
        let draw_end = (line_height / 2 + height as i32 / 2).min(height as i32 - 1);

        // Punto exacto donde el rayo toco la pared (0.0..1.0 a lo largo de la cara),
        // usado como coordenada horizontal de la textura.
        let wall_hit = if side == 0 {
            player.pos_y + perp_wall_dist * ray_dir_y
        } else {
            player.pos_x + perp_wall_dist * ray_dir_x
        };
        let wall_x = wall_hit - wall_hit.floor();
        let mut tex_x = (wall_x * TEX_SIZE as f64) as usize;
        // Evita que la textura salga espejada dependiendo de la cara/direccion del rayo.
        if (side == 0 && ray_dir_x > 0.0) || (side == 1 && ray_dir_y < 0.0) {
            tex_x = TEX_SIZE - 1 - tex_x;
        }
        tex_x = tex_x.min(TEX_SIZE - 1);

        // Caras "horizontales" (side 1) un poco mas oscuras que las "verticales" (side 0)
        // para que se note el contorno de las paredes; ademas se atenua con la distancia.
        let side_factor = if side == 1 { 0.7 } else { 1.0 };
        let distance_factor = (1.0 - (perp_wall_dist / MAX_SHADE_DISTANCE).min(1.0) * 0.88).max(0.12);
        let shade_factor = side_factor * distance_factor;

        let step_tex_y = TEX_SIZE as f64 / line_height.max(1) as f64;
        let mut tex_pos =
            (draw_start as f64 - height as f64 / 2.0 + line_height as f64 / 2.0) * step_tex_y;

        for y in draw_start..=draw_end {
            let tex_y = (tex_pos as usize).min(TEX_SIZE - 1);
            tex_pos += step_tex_y;

            let color = shade_color(texture_pixel(wall_id, tex_x, tex_y), shade_factor);
            buffer[y as usize * width + x] = color;
        }
    }
}
