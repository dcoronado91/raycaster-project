const METAL_DARK: u32 = 0x17_17_17;
const METAL_MID: u32 = 0x2C_2C_2C;
const METAL_HIGHLIGHT: u32 = 0x55_55_55;
const GRIP_COLOR: u32 = 0x2A_1F_14;
const GRIP_GRAIN: u32 = 0x40_30_1E;
const FLASH_COLOR: u32 = 0xE8_FF_9A; // destello verdoso, acorde a la paleta Matrix

// El arma se dibuja corrida a la derecha del centro, como en los shooters
// clasicos estilo Doom (se ve la mano/pistola desde el costado, no de frente).
const CX_OFFSET: i32 = 16;

fn put_pixel(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as usize, y as usize);
    if x < width && y < height {
        buffer[y * width + x] = color;
    }
}

fn fill_row(buffer: &mut [u32], width: usize, height: usize, cx: i32, y: i32, half_width: i32, color: u32) {
    for x in -half_width..=half_width {
        put_pixel(buffer, width, height, cx + x, y, color);
    }
}

/// Dibuja una pistola en primera persona (culata, guardamonte, corredera con
/// filo metalico, canon y mira) anclada a la parte inferior de la pantalla.
/// `flash_intensity` (0.0 a 1.0) controla el destello y el retroceso al
/// disparar; `bob_offset` es el balanceo vertical (caminar/reposo) en pixeles.
pub fn render(buffer: &mut [u32], width: usize, height: usize, flash_intensity: f64, bob_offset: f64) {
    let cx = width as i32 / 2 + CX_OFFSET;
    let recoil = (flash_intensity * 20.0) as i32;
    let base_y = height as i32 + bob_offset as i32 - recoil;

    // Cacha: se angosta hacia arriba, con veta de agarre a cuadros.
    for y in 0..80 {
        let half_width = 24 - (y * 8 / 80);
        for x in -half_width..=half_width {
            let grain = (y / 6 + x / 3) % 2 == 0;
            let color = if grain { GRIP_GRAIN } else { GRIP_COLOR };
            put_pixel(buffer, width, height, cx + x - 4, base_y - y, color);
        }
    }

    // Guardamonte: arco simple frente a la cacha.
    for step in 0..24 {
        let t = step as f64 / 23.0 * std::f64::consts::PI;
        let gx = (-20.0 * t.cos()) as i32;
        let gy = (66.0 + 14.0 * t.sin()) as i32;
        put_pixel(buffer, width, height, cx + gx - 16, base_y - gy, METAL_DARK);
    }

    // Corredera/armazon: cuerpo principal, con filo metalico claro arriba y
    // serraciones traseras para dar sensacion de metal real (no un bloque liso).
    for y in 78..152 {
        let half_width = 30 + ((y - 78) * 8 / 74);
        for x in -half_width..=half_width {
            let is_edge = y > 144;
            let is_notch = x > half_width - 16 && (y / 4) % 2 == 0;
            let color = if is_edge {
                METAL_HIGHLIGHT
            } else if is_notch {
                METAL_DARK
            } else {
                METAL_MID
            };
            put_pixel(buffer, width, height, cx + x, base_y - y, color);
        }
    }

    // Canon: mas angosto, sobresale al frente-arriba de la corredera.
    for y in 150..186 {
        fill_row(buffer, width, height, cx + 10, base_y - y, 13, METAL_DARK);
    }
    // Filo superior del canon, para que no se vea plano.
    for y in 178..186 {
        fill_row(buffer, width, height, cx + 10, base_y - y, 4, METAL_HIGHLIGHT);
    }

    // Mira frontal.
    for y in 184..193 {
        put_pixel(buffer, width, height, cx + 10, base_y - y, METAL_HIGHLIGHT);
    }

    if flash_intensity > 0.0 {
        let flash_radius = (26.0 * flash_intensity).round() as i32;
        let flash_cx = cx + 10;
        let flash_cy = base_y - 196;
        for dy in -flash_radius..=flash_radius {
            for dx in -flash_radius..=flash_radius {
                if dx * dx + dy * dy <= flash_radius * flash_radius {
                    put_pixel(buffer, width, height, flash_cx + dx, flash_cy + dy, FLASH_COLOR);
                }
            }
        }
    }
}
