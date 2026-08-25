const FONT_WIDTH: usize = 5;
const FONT_HEIGHT: usize = 7;

type Glyph = [&'static str; FONT_HEIGHT];

/// Fuente de pixeles 5x7 en mayusculas. Cada fila es una cadena de 5
/// caracteres ('#' = pixel encendido, '.' = apagado); un caracter sin
/// glifo definido se dibuja como espacio en blanco (nunca crashea).
fn glyph_for(ch: char) -> Glyph {
    match ch.to_ascii_uppercase() {
        'A' => [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        'B' => ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."],
        'C' => [".####", "#....", "#....", "#....", "#....", "#....", ".####"],
        'D' => ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."],
        'E' => ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
        'F' => ["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
        'G' => [".####", "#....", "#....", "#.###", "#...#", "#...#", ".####"],
        'H' => ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        'I' => ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "#####"],
        'J' => ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."],
        'K' => ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
        'L' => ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
        'M' => ["#...#", "##.##", "#.#.#", "#...#", "#...#", "#...#", "#...#"],
        'N' => ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"],
        'O' => [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        'P' => ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
        'Q' => [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"],
        'R' => ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
        'S' => [".####", "#....", "#....", ".###.", "....#", "....#", "####."],
        'T' => ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
        'U' => ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        'V' => ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
        'W' => ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"],
        'X' => ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"],
        'Y' => ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
        'Z' => ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"],
        '0' => [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."],
        '1' => ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", "#####"],
        '2' => [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"],
        '3' => [".###.", "#...#", "....#", "..##.", "....#", "#...#", ".###."],
        '4' => ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."],
        '5' => ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."],
        '6' => ["..##.", ".#...", "#....", "####.", "#...#", "#...#", ".###."],
        '7' => ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."],
        '8' => [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."],
        '9' => [".###.", "#...#", "#...#", ".####", "....#", "...#.", ".##.."],
        ':' => [".....", "..#..", ".....", ".....", ".....", "..#..", "....."],
        '!' => ["..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#.."],
        '.' => [".....", ".....", ".....", ".....", ".....", ".....", "..#.."],
        '-' => [".....", ".....", ".....", "#####", ".....", ".....", "....."],
        _ => [".....", ".....", ".....", ".....", ".....", ".....", "....."],
    }
}

fn put_pixel(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as usize, y as usize);
    if x < width && y < height {
        buffer[y * width + x] = color;
    }
}

/// Dibuja `text` (se interpreta en mayusculas) con la fuente 5x7, empezando
/// en (x, y), agrandando cada "pixel" del glifo `scale` veces.
pub fn draw_text(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, scale: i32, color: u32, text: &str) {
    let mut cursor_x = x;
    let advance = (FONT_WIDTH as i32 + 1) * scale;

    for ch in text.chars() {
        if ch != ' ' {
            let glyph = glyph_for(ch);
            for (row, line) in glyph.iter().enumerate() {
                for (col, pixel) in line.chars().enumerate() {
                    if pixel != '#' {
                        continue;
                    }
                    for sy in 0..scale {
                        for sx in 0..scale {
                            put_pixel(
                                buffer,
                                width,
                                height,
                                cursor_x + col as i32 * scale + sx,
                                y + row as i32 * scale + sy,
                                color,
                            );
                        }
                    }
                }
            }
        }
        cursor_x += advance;
    }
}

/// Ancho en pixeles que ocupa `text` dibujado con `draw_text` a esta escala.
pub fn text_width(text: &str, scale: i32) -> i32 {
    text.chars().count() as i32 * (FONT_WIDTH as i32 + 1) * scale
}

/// Como `draw_text`, pero centrado horizontalmente en `center_x`.
pub fn draw_text_centered(buffer: &mut [u32], width: usize, height: usize, center_x: i32, y: i32, scale: i32, color: u32, text: &str) {
    let x = center_x - text_width(text, scale) / 2;
    draw_text(buffer, width, height, x, y, scale, color, text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_width_scales_with_length_and_scale() {
        assert_eq!(text_width("AB", 2), 2 * (FONT_WIDTH as i32 + 1) * 2);
        assert_eq!(text_width("", 2), 0);
    }

    #[test]
    fn draw_text_paints_the_requested_color() {
        let (width, height) = (40, 20);
        let mut buffer = vec![0u32; width * height];

        draw_text(&mut buffer, width, height, 0, 0, 1, 0xFFFFFF, "A");

        assert!(buffer.iter().any(|&p| p == 0xFFFFFF));
    }

    #[test]
    fn draw_text_with_only_unsupported_characters_paints_nothing() {
        let (width, height) = (40, 20);
        let mut buffer = vec![0u32; width * height];

        draw_text(&mut buffer, width, height, 0, 0, 1, 0xFFFFFF, "@@@");

        assert!(buffer.iter().all(|&p| p == 0));
    }

    #[test]
    fn draw_text_off_screen_does_not_panic() {
        let (width, height) = (10, 10);
        let mut buffer = vec![0u32; width * height];

        draw_text(&mut buffer, width, height, -500, -500, 5, 0xFFFFFF, "HELLO WORLD");
        draw_text(&mut buffer, width, height, 500, 500, 5, 0xFFFFFF, "HELLO WORLD");
    }
}
