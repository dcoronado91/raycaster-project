use crate::map::Map;
use crate::player::Player;
use crate::rng::SimpleRng;

const AGENT_RADIUS: f64 = 0.25;
const CHASE_SPEED: f64 = 2.2; // celdas por segundo; un poco mas lento que el jugador
const WANDER_SPEED: f64 = 1.0; // celdas por segundo mientras patrulla sin haber detectado al jugador
const DETECTION_RADIUS: f64 = 7.0;
const CONTACT_RADIUS: f64 = 0.75;
const RESPAWN_DELAY: f64 = 5.0; // segundos que tarda un Agente eliminado en volver
const WANDER_MIN_SECONDS: f64 = 1.5;
const WANDER_MAX_SECONDS: f64 = 3.5;

/// Un Agente que patrulla el laberinto (deambulando al azar cuando no hay
/// nadie cerca) y persigue al jugador en cuanto lo detecta. Al recibir un
/// disparo queda inactivo (invisible, sin perseguir ni tocar al jugador)
/// durante `RESPAWN_DELAY` segundos y luego reaparece en su punto de origen.
pub struct Agent {
    pub x: f64,
    pub y: f64,
    spawn_x: f64,
    spawn_y: f64,
    respawn_timer: f64,
    wander_dir_x: f64,
    wander_dir_y: f64,
    wander_timer: f64,
    rng: SimpleRng,
}

impl Agent {
    pub fn new(x: f64, y: f64) -> Self {
        // Semilla derivada de la posicion de aparicion para que cada Agente
        // deambule en un patron distinto (no todos en sincronia).
        let seed = ((x * 1000.0) as i64 as u32)
            .wrapping_mul(747_796_405)
            .wrapping_add((y * 1000.0) as i64 as u32)
            .wrapping_mul(2_891_336_453)
            .wrapping_add(1);

        let mut agent = Agent {
            x,
            y,
            spawn_x: x,
            spawn_y: y,
            respawn_timer: 0.0,
            wander_dir_x: 1.0,
            wander_dir_y: 0.0,
            wander_timer: 0.0,
            rng: SimpleRng::new(seed),
        };
        agent.pick_new_wander_direction();
        agent
    }

    fn pick_new_wander_direction(&mut self) {
        let angle = (self.rng.gen_range(360) as f64).to_radians();
        self.wander_dir_x = angle.cos();
        self.wander_dir_y = angle.sin();
        let extra = self.rng.gen_range(200) as f64 / 100.0; // 0.0..2.0
        self.wander_timer = WANDER_MIN_SECONDS + extra * (WANDER_MAX_SECONDS - WANDER_MIN_SECONDS) / 2.0;
    }

    /// Deambula en `wander_dir` hasta que se cumple el tiempo o choca contra
    /// una pared, momento en el que elige una direccion nueva al azar.
    fn wander(&mut self, map: &Map, dt: f64) {
        self.wander_timer -= dt;
        if self.wander_timer <= 0.0 {
            self.pick_new_wander_direction();
        }

        let before = (self.x, self.y);
        let step = WANDER_SPEED * dt;
        map.move_point(&mut self.x, &mut self.y, self.wander_dir_x * step, self.wander_dir_y * step, AGENT_RADIUS);

        if (self.x, self.y) == before {
            self.pick_new_wander_direction();
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
    /// persigue al jugador solo si lo puede "ver" (dentro del radio de
    /// deteccion Y con linea de vision directa, no a traves de paredes);
    /// si no, deambula por el laberinto en vez de quedarse quieto
    /// vigilando, para que sea mas probable encontrarselo. Exigir linea de
    /// vision evita que un Agente "sepa" donde esta el jugador a traves de
    /// varias paredes y aparezca de sorpresa: si te puede perseguir, tambien
    /// lo puedes ver venir.
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

        let can_see_player = dist <= DETECTION_RADIUS
            && dist > 1e-6
            && map.has_line_of_sight(self.x, self.y, player.pos_x, player.pos_y);

        if !can_see_player {
            self.wander(map, dt);
            return;
        }

        let step = CHASE_SPEED * dt;
        map.move_point(&mut self.x, &mut self.y, dx / dist * step, dy / dist * step, AGENT_RADIUS);
    }

    /// El contacto exige estar cerca Y tener linea de vision directa: sin
    /// esto ultimo, un Agente y el jugador en pasillos perpendiculares
    /// podrian quedar mas cerca que `CONTACT_RADIUS` en linea recta a
    /// traves de la esquina de un pilar, aunque una pared los separe.
    pub fn is_touching_player(&self, player: &Player, map: &Map) -> bool {
        if !self.is_active() {
            return false;
        }
        let dx = self.x - player.pos_x;
        let dy = self.y - player.pos_y;
        let close_enough = (dx * dx + dy * dy).sqrt() < CONTACT_RADIUS;
        close_enough && map.has_line_of_sight(self.x, self.y, player.pos_x, player.pos_y)
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
    fn agent_wanders_instead_of_standing_still_when_player_is_out_of_range() {
        let map = Map::test_room();
        let mut agent = Agent::new(12.0, 12.0); // centro abierto del cuarto de pruebas
        let player = Player::new(0.0, 0.0); // fuera del radio de deteccion

        let mut moved = false;
        for _ in 0..50 {
            agent.update(&map, &player, 0.1);
            if (agent.x, agent.y) != (12.0, 12.0) {
                moved = true;
                break;
            }
        }

        assert!(moved, "el agente deberia deambular cuando no detecta al jugador");
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
        let map = Map::test_room();
        let mut agent = Agent::new(10.0, 10.0);
        let player = Player::new(10.1, 10.0);
        assert!(agent.is_touching_player(&player, &map));

        agent.hit();

        assert!(!agent.is_active());
        assert!(!agent.is_touching_player(&player, &map));
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
        let map = Map::test_room();
        let player = Player::new(10.0, 10.0);
        let close_agent = Agent::new(10.2, 10.0);
        let far_agent = Agent::new(15.0, 10.0);

        assert!(close_agent.is_touching_player(&player, &map));
        assert!(!far_agent.is_touching_player(&player, &map));
    }

    #[test]
    fn is_touching_player_is_false_across_a_pillar_even_if_close() {
        let map = Map::test_room();
        // Ambos puntos estan pegados a la esquina noroeste del pilar de
        // piedra (3..6, 3..6), uno por el lado oeste y otro por el norte:
        // en linea recta quedan a centesimas de distancia, pero el pilar
        // los separa por completo.
        let player = Player::new(2.99, 3.01);
        let agent = Agent::new(3.01, 2.99);

        assert!(!agent.is_touching_player(&player, &map));
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
