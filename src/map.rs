pub const WALL_NONE: u8 = 0;
pub const WALL_CONCRETE: u8 = 1; // corredores de concreto desgastado
pub const WALL_SERVER: u8 = 2; // bloques de servidores/racks
pub const WALL_CIRCUIT: u8 = 3; // paneles con cableado/conductos
pub const WALL_CODE: u8 = 4; // paneles con "codigo" verde brillante
pub const TILE_EXIT: u8 = 5; // casilla de salida del laberinto (no es pared)

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub player_spawn: (f64, f64),
    pub agent_spawns: Vec<(f64, f64)>,
    pub exit: (f64, f64),
    cells: Vec<u8>,
}

/// Generador de numeros pseudoaleatorios minimo (xorshift32), solo para
/// darle forma al laberinto de manera reproducible sin depender de la
/// crate `rand`. La misma semilla siempre produce el mismo laberinto.
struct SimpleRng(u32);

impl SimpleRng {
    fn new(seed: u32) -> Self {
        SimpleRng(seed.max(1))
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    fn gen_range(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n
    }
}

impl Map {
    /// Devuelve el id de casilla en (x, y). Fuera de rango se trata como
    /// pared solida (limite del mundo), asi el raycaster no necesita
    /// chequear bordes por separado.
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return WALL_CONCRETE;
        }
        self.cells[y as usize * self.width + x as usize]
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        matches!(self.get(x, y), WALL_CONCRETE | WALL_SERVER | WALL_CIRCUIT | WALL_CODE)
    }

    pub fn is_exit(&self, x: f64, y: f64) -> bool {
        self.get(x as i32, y as i32) == TILE_EXIT
    }

    /// Intenta desplazar un punto (x, y) por (dx, dy), deslizando sobre las
    /// paredes: cada eje se resuelve por separado para no trabar el
    /// movimiento diagonal contra una esquina. La usan tanto el jugador
    /// como los agentes que lo persiguen.
    pub fn move_point(&self, x: &mut f64, y: &mut f64, dx: f64, dy: f64, radius: f64) {
        let new_x = *x + dx;
        if !self.is_wall((new_x + dx.signum() * radius) as i32, *y as i32) {
            *x = new_x;
        }

        let new_y = *y + dy;
        if !self.is_wall(*x as i32, (new_y + dy.signum() * radius) as i32) {
            *y = new_y;
        }
    }

    /// Los 3 niveles del juego: laberintos reales generados con recursive
    /// backtracker (garantizado conectado de principio a fin), cada uno mas
    /// grande y con mas Agentes que el anterior.
    pub fn level(index: usize) -> Self {
        match index {
            0 => Self::generate(7, 7, 0x5EED_0001, WALL_CONCRETE, 2),
            1 => Self::generate(9, 9, 0x5EED_0002, WALL_SERVER, 4),
            _ => Self::generate(11, 11, 0x5EED_0003, WALL_CODE, 6),
        }
    }

    #[allow(dead_code)] // se usara en la pantalla de seleccion de nivel (proximo commit)
    pub fn level_count() -> usize {
        3
    }

    fn generate(cols: usize, rows: usize, seed: u32, wall_id: u8, agent_count: usize) -> Self {
        let width = cols * 2 + 1;
        let height = rows * 2 + 1;
        let mut cells = Self::carve_maze(cols, rows, width, height, seed, wall_id);

        let player_spawn = (1.5, 1.5);

        let exit_x = 2 * (cols - 1) + 1;
        let exit_y = 2 * (rows - 1) + 1;
        cells[exit_y * width + exit_x] = TILE_EXIT;
        let exit = (exit_x as f64 + 0.5, exit_y as f64 + 0.5);

        let agent_spawns = Self::pick_agent_spawns(cols, rows, agent_count);

        Map {
            width,
            height,
            player_spawn,
            agent_spawns,
            exit,
            cells,
        }
    }

    /// "Recursive backtracker": arranca en la celda (0,0), y en cada paso
    /// tumba la pared hacia una celda vecina sin visitar elegida al azar,
    /// retrocediendo (stack) cuando no quedan vecinas libres. El resultado
    /// es un arbol de expansion sobre la grilla: por construccion, siempre
    /// hay un unico camino entre dos celdas cualesquiera (nunca queda una
    /// zona inalcanzable).
    fn carve_maze(cols: usize, rows: usize, width: usize, height: usize, seed: u32, wall_id: u8) -> Vec<u8> {
        let mut cells = vec![wall_id; width * height];
        let mut visited = vec![false; cols * rows];
        let mut rng = SimpleRng::new(seed);
        let mut stack = vec![(0usize, 0usize)];

        visited[0] = true;
        Self::open_cell(&mut cells, width, 0, 0);

        while let Some(&(cx, cy)) = stack.last() {
            let mut neighbors: Vec<(usize, usize, usize, usize)> = Vec::new(); // (nx, ny, wall_x, wall_y)
            if cx > 0 && !visited[cy * cols + (cx - 1)] {
                neighbors.push((cx - 1, cy, 2 * cx, 2 * cy + 1));
            }
            if cx + 1 < cols && !visited[cy * cols + (cx + 1)] {
                neighbors.push((cx + 1, cy, 2 * cx + 2, 2 * cy + 1));
            }
            if cy > 0 && !visited[(cy - 1) * cols + cx] {
                neighbors.push((cx, cy - 1, 2 * cx + 1, 2 * cy));
            }
            if cy + 1 < rows && !visited[(cy + 1) * cols + cx] {
                neighbors.push((cx, cy + 1, 2 * cx + 1, 2 * cy + 2));
            }

            // neighbors.len().max(1) evita el modulo por cero de gen_range cuando
            // no queda ninguna vecina libre; en ese caso .get(0) sobre un Vec
            // vacio da None y simplemente se retrocede en el stack.
            let Some(&(nx, ny, wall_x, wall_y)) = neighbors.get(rng.gen_range(neighbors.len().max(1))) else {
                stack.pop();
                continue;
            };

            cells[wall_y * width + wall_x] = WALL_NONE;
            Self::open_cell(&mut cells, width, nx, ny);
            visited[ny * cols + nx] = true;
            stack.push((nx, ny));
        }

        cells
    }

    fn open_cell(cells: &mut [u8], width: usize, cx: usize, cy: usize) {
        let (x, y) = (cx * 2 + 1, cy * 2 + 1);
        cells[y * width + x] = WALL_NONE;
    }

    /// Puntos de aparicion de Agentes, repartidos en celdas del laberinto
    /// bien separadas entre si (todas garantizadas transitables, porque el
    /// generador visita cada celda de la grilla).
    fn pick_agent_spawns(cols: usize, rows: usize, count: usize) -> Vec<(f64, f64)> {
        let candidates = [
            (cols.saturating_sub(1), 0),          // esquina superior derecha
            (0, rows.saturating_sub(1)),          // esquina inferior izquierda
            (cols / 2, rows / 2),                 // centro
            (cols / 2, 0),                        // borde superior
            (0, rows / 2),                        // borde izquierdo
            (cols.saturating_sub(1), rows / 2),   // borde derecho
        ];
        candidates
            .iter()
            .take(count)
            .map(|&(cx, cy)| (cx as f64 * 2.0 + 1.5, cy as f64 * 2.0 + 1.5))
            .collect()
    }

    #[allow(dead_code)] // utilidad de depuracion
    pub fn debug_print(&self) {
        for y in 0..self.height {
            let mut line = String::with_capacity(self.width);
            for x in 0..self.width {
                let ch = match self.get(x as i32, y as i32) {
                    WALL_NONE => '.',
                    WALL_CONCRETE => '#',
                    WALL_SERVER => 'S',
                    WALL_CIRCUIT => 'K',
                    WALL_CODE => 'X',
                    TILE_EXIT => 'E',
                    _ => '?',
                };
                line.push(ch);
            }
            println!("{line}");
        }
    }

    #[allow(dead_code)] // solo la usa test_room(), bajo cfg(test)
    fn fill_rect(cells: &mut [u8], width: usize, x0: usize, y0: usize, w: usize, h: usize, wall: u8) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                cells[y * width + x] = wall;
            }
        }
    }

    /// Mapa simple y fijo (cuarto con 4 pilares), usado unicamente por los
    /// tests de colision de Player/Agent; no es contenido real del juego,
    /// asi esos tests no dependen de la forma exacta de los laberintos.
    #[cfg(test)]
    pub(crate) fn test_room() -> Self {
        let width = 24;
        let height = 24;
        let mut cells = vec![WALL_NONE; width * height];

        for x in 0..width {
            cells[x] = WALL_CONCRETE;
            cells[(height - 1) * width + x] = WALL_CONCRETE;
        }
        for y in 0..height {
            cells[y * width] = WALL_CONCRETE;
            cells[y * width + (width - 1)] = WALL_CONCRETE;
        }

        Self::fill_rect(&mut cells, width, 3, 3, 3, 3, WALL_SERVER);
        Self::fill_rect(&mut cells, width, 18, 3, 3, 3, WALL_CIRCUIT);
        Self::fill_rect(&mut cells, width, 3, 18, 3, 3, WALL_CODE);
        Self::fill_rect(&mut cells, width, 18, 18, 3, 3, WALL_SERVER);

        Map {
            width,
            height,
            player_spawn: (12.0, 12.0),
            agent_spawns: vec![],
            exit: (12.0, 12.0),
            cells,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn can_reach(map: &Map, from: (f64, f64), to: (f64, f64)) -> bool {
        let start = (from.0 as i32, from.1 as i32);
        let target = (to.0 as i32, to.1 as i32);

        let mut visited = vec![false; map.width * map.height];
        let mut queue = VecDeque::new();
        visited[start.1 as usize * map.width + start.0 as usize] = true;
        queue.push_back(start);

        while let Some((x, y)) = queue.pop_front() {
            if (x, y) == target {
                return true;
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx as usize >= map.width || ny as usize >= map.height {
                    continue;
                }
                let idx = ny as usize * map.width + nx as usize;
                if visited[idx] || map.is_wall(nx, ny) {
                    continue;
                }
                visited[idx] = true;
                queue.push_back((nx, ny));
            }
        }
        false
    }

    #[test]
    fn there_are_three_levels() {
        assert_eq!(Map::level_count(), 3);
    }

    #[test]
    fn every_level_has_a_solid_border() {
        for i in 0..Map::level_count() {
            let map = Map::level(i);
            for x in 0..map.width as i32 {
                assert!(map.is_wall(x, 0), "nivel {i}: borde superior");
                assert!(map.is_wall(x, map.height as i32 - 1), "nivel {i}: borde inferior");
            }
            for y in 0..map.height as i32 {
                assert!(map.is_wall(0, y), "nivel {i}: borde izquierdo");
                assert!(map.is_wall(map.width as i32 - 1, y), "nivel {i}: borde derecho");
            }
        }
    }

    #[test]
    fn every_level_player_spawn_is_walkable() {
        for i in 0..Map::level_count() {
            let map = Map::level(i);
            assert!(!map.is_wall(map.player_spawn.0 as i32, map.player_spawn.1 as i32), "nivel {i}");
        }
    }

    #[test]
    fn every_level_exit_tile_is_marked_and_reachable_from_spawn() {
        for i in 0..Map::level_count() {
            let map = Map::level(i);
            assert!(map.is_exit(map.exit.0, map.exit.1), "nivel {i}: la salida no quedo marcada");
            assert!(
                can_reach(&map, map.player_spawn, map.exit),
                "nivel {i}: la salida no es alcanzable desde el spawn del jugador"
            );
        }
    }

    #[test]
    fn every_level_has_the_expected_agent_count_all_walkable() {
        let expected_counts = [2, 4, 6];
        for i in 0..Map::level_count() {
            let map = Map::level(i);
            assert_eq!(map.agent_spawns.len(), expected_counts[i], "nivel {i}");
            for &(x, y) in &map.agent_spawns {
                assert!(!map.is_wall(x as i32, y as i32), "nivel {i}: spawn de agente dentro de una pared");
            }
        }
    }

    #[test]
    fn out_of_bounds_is_treated_as_solid() {
        let map = Map::level(0);
        assert!(map.is_wall(-1, 5));
        assert!(map.is_wall(5, -1));
        assert!(map.is_wall(map.width as i32, 5));
        assert!(map.is_wall(5, map.height as i32));
    }
}
