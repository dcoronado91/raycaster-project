pub const WALL_NONE: u8 = 0;
pub const WALL_BRICK: u8 = 1;
pub const WALL_STONE: u8 = 2;
pub const WALL_WOOD: u8 = 3;
pub const WALL_MOSS: u8 = 4;

pub struct Map {
    pub width: usize,
    pub height: usize,
    cells: Vec<u8>,
}

impl Map {
    /// Devuelve el id de pared en (x, y). Fuera de rango se trata como pared solida
    /// (limite del mundo), asi el raycaster no necesita chequear bordes por separado.
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return WALL_BRICK;
        }
        self.cells[y as usize * self.width + x as usize]
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        self.get(x, y) != WALL_NONE
    }

    pub fn level_1() -> Self {
        let width = 24;
        let height = 24;
        let mut cells = vec![WALL_NONE; width * height];

        for x in 0..width {
            cells[x] = WALL_BRICK;
            cells[(height - 1) * width + x] = WALL_BRICK;
        }
        for y in 0..height {
            cells[y * width] = WALL_BRICK;
            cells[y * width + (width - 1)] = WALL_BRICK;
        }

        Self::fill_rect(&mut cells, width, 3, 3, 3, 3, WALL_STONE);
        Self::fill_rect(&mut cells, width, 18, 3, 3, 3, WALL_WOOD);
        Self::fill_rect(&mut cells, width, 3, 18, 3, 3, WALL_MOSS);
        Self::fill_rect(&mut cells, width, 18, 18, 3, 3, WALL_STONE);

        Map {
            width,
            height,
            cells,
        }
    }

    fn fill_rect(cells: &mut [u8], width: usize, x0: usize, y0: usize, w: usize, h: usize, wall: u8) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                cells[y * width + x] = wall;
            }
        }
    }

    pub fn debug_print(&self) {
        for y in 0..self.height {
            let mut line = String::with_capacity(self.width);
            for x in 0..self.width {
                let ch = match self.get(x as i32, y as i32) {
                    WALL_NONE => '.',
                    WALL_BRICK => '#',
                    WALL_STONE => 'S',
                    WALL_WOOD => 'W',
                    WALL_MOSS => 'M',
                    _ => '?',
                };
                line.push(ch);
            }
            println!("{line}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_1_has_expected_dimensions() {
        let map = Map::level_1();
        assert_eq!(map.width, 24);
        assert_eq!(map.height, 24);
    }

    #[test]
    fn level_1_border_is_solid() {
        let map = Map::level_1();
        for x in 0..map.width as i32 {
            assert!(map.is_wall(x, 0));
            assert!(map.is_wall(x, map.height as i32 - 1));
        }
        for y in 0..map.height as i32 {
            assert!(map.is_wall(0, y));
            assert!(map.is_wall(map.width as i32 - 1, y));
        }
    }

    #[test]
    fn level_1_interior_open_space_is_walkable() {
        let map = Map::level_1();
        assert!(!map.is_wall(12, 12));
    }

    #[test]
    fn level_1_pillars_have_distinct_wall_types() {
        let map = Map::level_1();
        assert_eq!(map.get(4, 4), WALL_STONE);
        assert_eq!(map.get(19, 4), WALL_WOOD);
        assert_eq!(map.get(4, 19), WALL_MOSS);
        assert_eq!(map.get(19, 19), WALL_STONE);
    }

    #[test]
    fn out_of_bounds_is_treated_as_solid() {
        let map = Map::level_1();
        assert!(map.is_wall(-1, 5));
        assert!(map.is_wall(5, -1));
        assert!(map.is_wall(map.width as i32, 5));
        assert!(map.is_wall(5, map.height as i32));
    }
}
