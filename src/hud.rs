use crate::text;

const BULLET_WIDTH: i32 = 10;
const BULLET_HEIGHT: i32 = 24;
const BULLET_GAP: i32 = 6;
const MARGIN: i32 = 16;
const PANEL_PADDING: i32 = 6;

const LOADED_BODY: u32 = 0xC9_A2_27;
const LOADED_TIP: u32 = 0xE8_C2_4A;
const EMPTY_BODY: u32 = 0x2A_2A_2A;
const EMPTY_TIP: u32 = 0x1A_1A_1A;
const PANEL_COLOR: u32 = 0x00_05_00;
const RELOADING_COLOR: u32 = 0xFF_FF_66;

fn put_pixel(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as usize, y as usize);
    if x < width && y < height {
        buffer[y * width + x] = color;
    }
}

/// Un cartucho: cuerpo rectangular y una punta que se angosta hacia arriba.
fn draw_bullet(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, loaded: bool) {
    let (body, tip) = if loaded { (LOADED_BODY, LOADED_TIP) } else { (EMPTY_BODY, EMPTY_TIP) };

    for dy in 8..BULLET_HEIGHT {
        for dx in 0..BULLET_WIDTH {
            put_pixel(buffer, width, height, x + dx, y + dy, body);
        }
    }
    for dy in 0..8 {
        let inset = dy / 2;
        for dx in inset..(BULLET_WIDTH - inset) {
            put_pixel(buffer, width, height, x + dx, y + dy, tip);
        }
    }
}

/// Dibuja la municion restante (de `max_ammo`) como una fila de cartuchos en
/// la esquina inferior derecha de la pantalla; los gastados se ven oscuros.
/// Mientras `reloading` es true, se muestra un aviso "RECARGANDO" encima.
pub fn draw_ammo(buffer: &mut [u32], width: usize, height: usize, ammo: u32, max_ammo: u32, reloading: bool) {
    if max_ammo == 0 {
        return;
    }

    let total_width = max_ammo as i32 * BULLET_WIDTH + (max_ammo as i32 - 1) * BULLET_GAP;
    let origin_x = width as i32 - MARGIN - total_width;
    let origin_y = height as i32 - MARGIN - BULLET_HEIGHT;

    for y in -PANEL_PADDING..BULLET_HEIGHT + PANEL_PADDING {
        for x in -PANEL_PADDING..total_width + PANEL_PADDING {
            put_pixel(buffer, width, height, origin_x + x, origin_y + y, PANEL_COLOR);
        }
    }

    for i in 0..max_ammo {
        let x = origin_x + i as i32 * (BULLET_WIDTH + BULLET_GAP);
        draw_bullet(buffer, width, height, x, origin_y, i < ammo);
    }

    if reloading {
        text::draw_text(buffer, width, height, origin_x, origin_y - 20, 1, RELOADING_COLOR, "RECARGANDO");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_ammo_paints_something_and_does_not_panic() {
        let (width, height) = (800, 600);
        let mut buffer = vec![0u32; width * height];

        draw_ammo(&mut buffer, width, height, 6, 6, false);
        assert!(buffer.iter().any(|&p| p != 0));
    }

    #[test]
    fn draw_ammo_handles_empty_and_reloading_without_panicking() {
        let (width, height) = (800, 600);
        let mut buffer = vec![0u32; width * height];

        draw_ammo(&mut buffer, width, height, 0, 6, true);
        assert!(buffer.iter().any(|&p| p != 0));
    }

    #[test]
    fn draw_ammo_with_zero_max_ammo_does_not_panic() {
        let (width, height) = (800, 600);
        let mut buffer = vec![0u32; width * height];

        draw_ammo(&mut buffer, width, height, 0, 0, false);
    }
}
