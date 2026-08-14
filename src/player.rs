use crate::map::Map;

/// Radio de colision del jugador, en celdas de mapa. Evita que el jugador
/// pegue la camara contra la textura de la pared al chocar de frente.
const COLLISION_RADIUS: f64 = 0.2;

pub struct Player {
    pub pos_x: f64,
    pub pos_y: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub plane_x: f64,
    pub plane_y: f64,
}

impl Player {
    /// Jugador mirando hacia "arriba" (y decreciente) con un FOV de ~66 grados.
    /// dir y plane siempre deben ser perpendiculares entre si.
    pub fn new(pos_x: f64, pos_y: f64) -> Self {
        Player {
            pos_x,
            pos_y,
            dir_x: 0.0,
            dir_y: -1.0,
            plane_x: 0.66,
            plane_y: 0.0,
        }
    }

    /// Rota la direccion de vista y el plano de camara `angle` radianes
    /// (positivo = sentido horario, usado para el mouse-look horizontal).
    pub fn rotate(&mut self, angle: f64) {
        let (sin_a, cos_a) = angle.sin_cos();

        let old_dir_x = self.dir_x;
        self.dir_x = self.dir_x * cos_a - self.dir_y * sin_a;
        self.dir_y = old_dir_x * sin_a + self.dir_y * cos_a;

        let old_plane_x = self.plane_x;
        self.plane_x = self.plane_x * cos_a - self.plane_y * sin_a;
        self.plane_y = old_plane_x * sin_a + self.plane_y * cos_a;
    }

    /// Intenta mover al jugador por (dx, dy). Cada eje se prueba por separado
    /// contra el mapa para poder "deslizar" sobre una pared en vez de
    /// quedar trabado cuando el movimiento no es perfectamente perpendicular.
    pub fn try_move(&mut self, map: &Map, dx: f64, dy: f64) {
        let new_x = self.pos_x + dx;
        if !map.is_wall((new_x + dx.signum() * COLLISION_RADIUS) as i32, self.pos_y as i32) {
            self.pos_x = new_x;
        }

        let new_y = self.pos_y + dy;
        if !map.is_wall(self.pos_x as i32, (new_y + dy.signum() * COLLISION_RADIUS) as i32) {
            self.pos_y = new_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_keeps_dir_and_plane_perpendicular_and_unit_length() {
        let mut player = Player::new(5.0, 5.0);
        player.rotate(0.7);

        let dir_len = (player.dir_x * player.dir_x + player.dir_y * player.dir_y).sqrt();
        assert!((dir_len - 1.0).abs() < 1e-9);

        let dot = player.dir_x * player.plane_x + player.dir_y * player.plane_y;
        assert!(dot.abs() < 1e-9);
    }

    #[test]
    fn try_move_is_blocked_by_a_wall() {
        let map = Map::level_1();
        let mut player = Player::new(4.5, 2.0); // justo arriba del pilar de piedra (filas/cols 3..6)

        player.try_move(&map, 0.0, 1.0); // intenta moverse hacia el pilar

        assert!(player.pos_y < 3.0, "el jugador no deberia atravesar la pared");
    }

    #[test]
    fn try_move_slides_along_open_space() {
        let map = Map::level_1();
        let mut player = Player::new(12.0, 12.0); // centro abierto del mapa

        player.try_move(&map, 0.5, 0.0);

        assert!((player.pos_x - 12.5).abs() < 1e-9);
        assert!((player.pos_y - 12.0).abs() < 1e-9);
    }
}
