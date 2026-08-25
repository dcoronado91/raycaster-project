use crate::map::Map;
use crate::player::Player;

const AGENT_RADIUS: f64 = 0.25;
const CHASE_SPEED: f64 = 2.2; // celdas por segundo; un poco mas lento que el jugador
const DETECTION_RADIUS: f64 = 9.0;
const CONTACT_RADIUS: f64 = 0.5;
const RESPAWN_DELAY: f64 = 5.0; // segundos que tarda un Agente eliminado en volver

/// Un Agente que patrulla el laberinto y persigue al jugador cuando lo
/// detecta. Al recibir un disparo queda inactivo (invisible, sin perseguir
/// ni tocar al jugador) durante `RESPAWN_DELAY` segundos y luego reaparece
/// en su punto de origen.
pub struct Agent {
    pub x: f64,
    pub y: f64,
    spawn_x: f64,
    spawn_y: f64,
    respawn_timer: f64,
}

impl Agent {
    pub fn new(x: f64, y: f64) -> Self {
        Agent {
            x,
            y,
            spawn_x: x,
            spawn_y: y,
            respawn_timer: 0.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.respawn_timer <= 0.0
    }

    /// Marca al agente como eliminado por un disparo; vuelve a aparecer
    /// pasado `RESPAWN_DELAY` en `update`.
    pub fn hit(&mut self) {
        self.respawn_timer = RESPAWN_DELAY;
    }

    /// Mientras esta inactivo, cuenta la espera de reaparicion. Activo,
    /// persigue al jugador si esta dentro del radio de deteccion (deslizando
    /// sobre las paredes); si no, se queda quieto vigilando.
    pub fn update(&mut self, map: &Map, player: &Player, dt: f64) {
        if !self.is_active() {
            self.respawn_timer -= dt;
            if self.respawn_timer <= 0.0 {
                self.x = self.spawn_x;
                self.y = self.spawn_y;
            }
            return;
        }

        let dx = player.pos_x - self.x;
        let dy = player.pos_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > DETECTION_RADIUS || dist < 1e-6 {
            return;
        }

        let step = CHASE_SPEED * dt;
        map.move_point(&mut self.x, &mut self.y, dx / dist * step, dy / dist * step, AGENT_RADIUS);
    }

    pub fn is_touching_player(&self, player: &Player) -> bool {
        if !self.is_active() {
            return false;
        }
        let dx = self.x - player.pos_x;
        let dy = self.y - player.pos_y;
        (dx * dx + dy * dy).sqrt() < CONTACT_RADIUS
    }
}

/// Color de un texel (tx, ty) del Agente (traje negro, gafas oscuras), o
/// `None` si es transparente. `walk_phase` alterna entre dos poses para dar
/// un ciclo de caminata simple tipo tijera en brazos y piernas.
pub fn agent_pixel(tx: usize, ty: usize, walk_phase: i32) -> Option<u32> {
    const CENTER: f64 = 32.0;
    let fx = tx as f64 - CENTER;
    let swing = if walk_phase % 2 == 0 { 1.0 } else { -1.0 };

    // Cabeza: pelo arriba/laterales, gafas oscuras en franja central, piel en el resto.
    if (4..18).contains(&ty) {
        if fx.abs() > 8.0 {
            return None;
        }
        if (10..14).contains(&ty) {
            return Some(0x0A_0A_0A);
        }
        if ty < 7 || fx.abs() > 6.0 {
            return Some(0x1A_12_0A);
        }
        return Some(0xE0_B8_8C);
    }

    // Cuello.
    if (18..21).contains(&ty) {
        if fx.abs() <= 5.0 {
            return Some(0xE0_B8_8C);
        }
        return None;
    }

    // Torso: saco negro, mangas, cuello de camisa blanco y corbata; los
    // brazos se separan un poco segun `swing` para simular el vaiven al caminar.
    if (21..45).contains(&ty) {
        let arm_span = 16.0 + swing * 2.0;
        if fx.abs() > arm_span {
            return None;
        }
        if fx.abs() > 12.0 {
            return Some(0x14_14_14);
        }
        if (21..24).contains(&ty) && fx.abs() <= 4.0 {
            return Some(0xF5_F5_F5);
        }
        if fx.abs() <= 1.5 {
            return Some(0x0A_0A_0A);
        }
        return Some(0x1A_1A_1A);
    }

    // Piernas: tijera simple, una pierna se adelanta y la otra se atrasa.
    if ty >= 45 {
        let leg_shift = swing * 3.0;
        let left_leg = (-13.0 - leg_shift..=-2.0 - leg_shift).contains(&fx);
        let right_leg = (2.0 + leg_shift..=13.0 + leg_shift).contains(&fx);
        if left_leg || right_leg {
            return Some(0x0F_0F_0F);
        }
        return None;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    #[test]
    fn agent_ignores_player_outside_detection_radius() {
        let map = Map::test_room();
        let mut agent = Agent::new(2.0, 2.0);
        let player = Player::new(20.0, 20.0); // muy lejos

        agent.update(&map, &player, 1.0);

        assert_eq!(agent.x, 2.0);
        assert_eq!(agent.y, 2.0);
    }

    #[test]
    fn agent_chases_player_within_detection_radius() {
        let map = Map::test_room();
        let mut agent = Agent::new(12.0, 12.0);
        let player = Player::new(12.0, 16.0); // a 4 celdas, en espacio abierto

        agent.update(&map, &player, 0.5);

        assert!(agent.y > 12.0, "el agente deberia avanzar hacia el jugador");
    }

    #[test]
    fn hit_agent_becomes_inactive_and_stops_touching_player() {
        let mut agent = Agent::new(10.0, 10.0);
        let player = Player::new(10.1, 10.0);
        assert!(agent.is_touching_player(&player));

        agent.hit();

        assert!(!agent.is_active());
        assert!(!agent.is_touching_player(&player));
    }

    #[test]
    fn inactive_agent_respawns_at_its_origin_after_the_delay() {
        let map = Map::test_room();
        let player = Player::new(0.0, 0.0); // lejos, no afecta el respawn
        let mut agent = Agent::new(5.0, 5.0);

        agent.hit();
        agent.update(&map, &player, 4.0); // aun no pasa el tiempo de espera
        assert!(!agent.is_active());

        agent.update(&map, &player, 2.0); // ya se cumplio RESPAWN_DELAY (5.0s)
        assert!(agent.is_active());
        assert_eq!((agent.x, agent.y), (5.0, 5.0));
    }

    #[test]
    fn is_touching_player_detects_close_contact() {
        let player = Player::new(10.0, 10.0);
        let close_agent = Agent::new(10.2, 10.0);
        let far_agent = Agent::new(15.0, 10.0);

        assert!(close_agent.is_touching_player(&player));
        assert!(!far_agent.is_touching_player(&player));
    }

    #[test]
    fn agent_pixel_head_region_has_sunglasses_band() {
        assert_eq!(agent_pixel(32, 12, 0), Some(0x0A_0A_0A));
    }

    #[test]
    fn agent_pixel_far_corner_is_transparent() {
        assert_eq!(agent_pixel(0, 0, 0), None);
    }
}
