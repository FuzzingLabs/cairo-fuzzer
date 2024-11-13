use std::cell::Cell;
use std::ops::RangeInclusive;

/// Random number generator implementation using xorshift64
pub struct Rng {
    /// Internal xorshift seed
    seed: Cell<u64>,
}

impl Rng {
    /// Creates a RNG with a fixed `seed` value
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

    /// Generate a random number in the range [start, end]
    pub fn gen_range(&self, range: RangeInclusive<usize>) -> usize {
        let start = *range.start();
        let end = *range.end();
        assert!(end >= start, "end must be greater than or equal to start");
        start + self.rand_usize() % (end - start + 1)
    }
}
