use crate::player::Player;
use crate::raycaster::shade_color;

const TEX_SIZE: usize = 64;
const MAX_SHADE_DISTANCE: f64 = 10.0;

/// Proyecta un punto del mundo al espacio de camara del jugador.
/// Devuelve `None` si el punto queda detras (o encima) de la camara.
fn project_to_camera_space(player: &Player, world_x: f64, world_y: f64) -> Option<(f64, f64)> {
    let rel_x = world_x - player.pos_x;
    let rel_y = world_y - player.pos_y;

    let inv_det = 1.0 / (player.plane_x * player.dir_y - player.dir_x * player.plane_y);
    let transform_x = inv_det * (player.dir_y * rel_x - player.dir_x * rel_y);
    let transform_y = inv_det * (-player.plane_y * rel_x + player.plane_x * rel_y);

    if transform_y <= 0.1 {
        None
    } else {
        Some((transform_x, transform_y))
    }
}

/// Dibuja un sprite tipo "billboard" en (world_x, world_y): siempre queda de
/// frente a la camara y se recorta contra las paredes usando el z-buffer que
/// dejo `raycaster::render`. `pixel_fn(tex_x, tex_y)` decide el color de
/// cada texel (o `None` si es transparente), asi este modulo no sabe nada
/// de que criatura o decoracion esta dibujando.
pub fn render(
    buffer: &mut [u32],
    z_buffer: &[f64],
    width: usize,
    height: usize,
    player: &Player,
    world_x: f64,
    world_y: f64,
    pixel_fn: impl Fn(usize, usize) -> Option<u32>,
) {
    let Some((transform_x, transform_y)) = project_to_camera_space(player, world_x, world_y)
    else {
        return;
    };

    let sprite_screen_x = ((width as f64 / 2.0) * (1.0 + transform_x / transform_y)) as i32;

    let sprite_size = (height as f64 / transform_y).abs() as i32;
    let draw_start_y = (-sprite_size / 2 + height as i32 / 2).max(0);
    let draw_end_y = (sprite_size / 2 + height as i32 / 2).min(height as i32 - 1);
    let draw_start_x = (-sprite_size / 2 + sprite_screen_x).max(0);
    let draw_end_x = (sprite_size / 2 + sprite_screen_x).min(width as i32 - 1);

    let distance_factor = (1.0 - (transform_y / MAX_SHADE_DISTANCE).min(1.0) * 0.88).max(0.12);
    let sprite_left = -sprite_size / 2 + sprite_screen_x;
    let sprite_top = -sprite_size / 2 + height as i32 / 2;

    for stripe in draw_start_x..draw_end_x {
        if transform_y >= z_buffer[stripe as usize] {
            continue; // hay una pared mas cerca en esta columna: sprite oculto
        }

        let tex_x = (((stripe - sprite_left) * TEX_SIZE as i32) / sprite_size.max(1))
            .clamp(0, TEX_SIZE as i32 - 1) as usize;

        for y in draw_start_y..=draw_end_y {
            let tex_y = (((y - sprite_top) * TEX_SIZE as i32) / sprite_size.max(1))
                .clamp(0, TEX_SIZE as i32 - 1) as usize;

            if let Some(color) = pixel_fn(tex_x, tex_y) {
                buffer[y as usize * width + stripe as usize] = shade_color(color, distance_factor);
            }
        }
    }
}

const CROSSHAIR_TOLERANCE_PX: i32 = 55;

/// Indica si (world_x, world_y) esta dentro de la mira central de la
/// pantalla, sin una pared por delante y dentro de `max_range`. Se usa
/// para resolver si un disparo le acierta a un objetivo.
pub fn is_targetable(
    player: &Player,
    world_x: f64,
    world_y: f64,
    z_buffer: &[f64],
    width: usize,
    max_range: f64,
) -> bool {
    let Some((transform_x, transform_y)) = project_to_camera_space(player, world_x, world_y)
    else {
        return false;
    };
    if transform_y > max_range {
        return false;
    }

    let screen_x = ((width as f64 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
    if (screen_x - width as i32 / 2).abs() > CROSSHAIR_TOLERANCE_PX {
        return false;
    }

    let center_col = (width / 2).min(z_buffer.len().saturating_sub(1));
    transform_y < z_buffer[center_col]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_directly_ahead_projects_to_screen_center() {
        let player = Player::new(0.0, 0.0);
        let (transform_x, transform_y) = project_to_camera_space(&player, 0.0, -5.0).unwrap();

        assert!(transform_x.abs() < 1e-9);
        assert!((transform_y - 5.0).abs() < 1e-9);
    }

    #[test]
    fn sprite_behind_player_is_not_projected() {
        let player = Player::new(0.0, 0.0);
        assert!(project_to_camera_space(&player, 0.0, 5.0).is_none());
    }

    #[test]
    fn target_directly_ahead_with_clear_view_is_targetable() {
        let player = Player::new(0.0, 0.0);
        let z_buffer = vec![100.0; 800];

        assert!(is_targetable(&player, 0.0, -5.0, &z_buffer, 800, 12.0));
    }

    #[test]
    fn target_far_off_center_is_not_targetable() {
        let player = Player::new(0.0, 0.0);
        let z_buffer = vec![100.0; 800];

        assert!(!is_targetable(&player, 5.0, -1.0, &z_buffer, 800, 12.0));
    }

    #[test]
    fn target_behind_a_closer_wall_is_not_targetable() {
        let player = Player::new(0.0, 0.0);
        let mut z_buffer = vec![100.0; 800];
        z_buffer[400] = 2.0; // pared mas cerca que el objetivo, tapandolo

        assert!(!is_targetable(&player, 0.0, -5.0, &z_buffer, 800, 12.0));
    }

    #[test]
    fn target_beyond_max_range_is_not_targetable() {
        let player = Player::new(0.0, 0.0);
        let z_buffer = vec![100.0; 800];

        assert!(!is_targetable(&player, 0.0, -20.0, &z_buffer, 800, 12.0));
    }
}
