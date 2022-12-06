use std::cell::Cell;

/// Random number generator implementation using xorshift64
pub struct Rng {
    /// Interal xorshift seed
    seed: Cell<u64>,
}

impl Rng {
    /// Create a new, TSC-seeded random number generator
    pub fn new() -> Self {
        let ret = Rng {
            seed: Cell::new(unsafe { core::arch::x86_64::_rdtsc() }),
        };

        for _ in 0..1000 {
            let _ = ret.rand();
        }

        ret
    }

    /// Created a RNG with a fixed `seed` value
    pub fn seeded(seed: u64) -> Self {
        Rng {
            seed: Cell::new(seed),
        }
    }

    /// Get a random 64-bit number using xorshift
    pub fn rand(&self) -> usize {
        let mut seed = self.seed.get();
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 43;
        self.seed.set(seed);
        seed as usize
    }
}