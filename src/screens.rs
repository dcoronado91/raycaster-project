use crate::text;

const TITLE_COLOR: u32 = 0x00_FF_41;
const SUBTITLE_COLOR: u32 = 0x7A_C9_8C;
const HINT_COLOR: u32 = 0x3D_7A_4A;
const SELECTED_COLOR: u32 = 0xFF_FF_FF;
const UNSELECTED_COLOR: u32 = 0x2E_8B_57;
const DANGER_COLOR: u32 = 0xFF_33_33;

/// Hash entero barato (sin dependencias externas) usado para el "ruido"
/// del fondo: nada de aleatoriedad real, solo una mezcla de bits que se ve
/// suficientemente caotica pixel a pixel.
fn hash2(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

/// Fondo tipo "lluvia de codigo": franjas verticales de estatica verde que
/// caen con el tiempo, para que los menus no se sientan una pantalla vacia.
fn draw_background(buffer: &mut [u32], width: usize, height: usize, elapsed: f64) {
    let scroll = (elapsed * 45.0) as u32;
    for y in 0..height {
        let noise_y = (y as u32).wrapping_add(scroll);
        for x in 0..width {
            let h = hash2(x as u32 / 3, noise_y / 6);
            let color = match h % 211 {
                0..=1 => 0x00_FF_41,
                2..=6 => 0x0A_3A_16,
                _ => 0x00_05_00,
            };
            buffer[y * width + x] = color;
        }
    }
}

pub fn draw_welcome(buffer: &mut [u32], width: usize, height: usize, elapsed: f64, selected_level: usize, level_count: usize) {
    draw_background(buffer, width, height, elapsed);
    let cx = width as i32 / 2;

    text::draw_text_centered(buffer, width, height, cx, 60, 6, TITLE_COLOR, "RAYCASTER MATRIX");
    text::draw_text_centered(buffer, width, height, cx, 130, 2, SUBTITLE_COLOR, "ENCUENTRA LA SALIDA DEL LABERINTO");
    text::draw_text_centered(buffer, width, height, cx, 155, 2, SUBTITLE_COLOR, "LOS AGENTES TE CAZAN - DISPARA PARA SOBREVIVIR");

    for i in 0..level_count {
        let y = 240 + i as i32 * 45;
        let label = format!("NIVEL {}", i + 1);
        let color = if i == selected_level { SELECTED_COLOR } else { UNSELECTED_COLOR };
        let marker = if i == selected_level { "> " } else { "  " };
        text::draw_text_centered(buffer, width, height, cx, y, 4, color, &format!("{marker}{label}"));
    }

    text::draw_text_centered(buffer, width, height, cx, height as i32 - 70, 2, HINT_COLOR, "FLECHAS ARRIBA/ABAJO PARA ELEGIR");
    text::draw_text_centered(buffer, width, height, cx, height as i32 - 45, 2, HINT_COLOR, "ENTER PARA JUGAR - ESC PARA SALIR");
}

pub fn draw_game_over(buffer: &mut [u32], width: usize, height: usize, elapsed: f64) {
    draw_background(buffer, width, height, elapsed);
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    text::draw_text_centered(buffer, width, height, cx, cy - 70, 7, DANGER_COLOR, "TE ATRAPARON");
    text::draw_text_centered(buffer, width, height, cx, cy + 30, 3, SUBTITLE_COLOR, "ENTER PARA REINTENTAR");
    text::draw_text_centered(buffer, width, height, cx, cy + 60, 2, HINT_COLOR, "ESC PARA SALIR");
}

pub fn draw_success(buffer: &mut [u32], width: usize, height: usize, elapsed: f64, is_last_level: bool) {
    draw_background(buffer, width, height, elapsed);
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    text::draw_text_centered(buffer, width, height, cx, cy - 70, 6, TITLE_COLOR, "NIVEL COMPLETADO");
    let prompt = if is_last_level {
        "COMPLETASTE TODO - ENTER PARA EL MENU"
    } else {
        "ENTER PARA EL SIGUIENTE NIVEL"
    };
    text::draw_text_centered(buffer, width, height, cx, cy + 30, 3, SUBTITLE_COLOR, prompt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welcome_screen_paints_something_and_does_not_panic() {
        let (width, height) = (800, 600);
        let mut buffer = vec![0u32; width * height];
        draw_welcome(&mut buffer, width, height, 1.23, 1, 3);
        assert!(buffer.iter().any(|&p| p != 0));
    }

    #[test]
    fn game_over_screen_paints_something_and_does_not_panic() {
        let (width, height) = (800, 600);
        let mut buffer = vec![0u32; width * height];
        draw_game_over(&mut buffer, width, height, 4.5);
        assert!(buffer.iter().any(|&p| p != 0));
    }

    #[test]
    fn success_screen_paints_something_and_does_not_panic() {
        let (width, height) = (800, 600);
        let mut buffer = vec![0u32; width * height];
        draw_success(&mut buffer, width, height, 7.0, false);
        draw_success(&mut buffer, width, height, 7.0, true);
        assert!(buffer.iter().any(|&p| p != 0));
    }
}
