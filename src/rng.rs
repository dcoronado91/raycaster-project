/// Generador de numeros pseudoaleatorios minimo (xorshift32), usado tanto
/// para dar forma a los laberintos como para el deambular de los Agentes,
/// sin depender de la crate `rand`. La misma semilla siempre produce la
/// misma secuencia (reproducible, util para los tests).
pub struct SimpleRng(u32);

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        SimpleRng(seed.max(1))
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    pub fn gen_range(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_the_same_sequence() {
        let mut a = SimpleRng::new(42);
        let mut b = SimpleRng::new(42);
        for _ in 0..10 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn gen_range_stays_within_bounds() {
        let mut rng = SimpleRng::new(7);
        for _ in 0..200 {
            assert!(rng.gen_range(5) < 5);
        }
    }
}
