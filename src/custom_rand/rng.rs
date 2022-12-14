use std::cell::Cell;

/// Random number generator implementation using xorshift64
pub struct Rng {
    /// Interal xorshift seed
    seed: Cell<u64>,
}

impl Rng {
    /// Created a RNG with a fixed `seed` value
    pub fn seeded(seed: u64) -> Self {
        Rng {
            seed: Cell::new(seed),
        }
    }

    /// Get a random 64-bit number using xorshift
    pub fn rand(&self) -> u64 {
        let mut seed = self.seed.get();
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 43;
        self.seed.set(seed);
        seed
    }

    /// Get a random usize number using xorshift
    pub fn rand_usize(&self) -> usize {
        self.rand() as usize
    }
}
