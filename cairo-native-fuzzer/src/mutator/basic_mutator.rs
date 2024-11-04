use rand::Rng;
use starknet_types_core::felt::Felt;

pub struct Mutator {
    rng: rand::rngs::ThreadRng,
}

impl Mutator {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    pub fn mutate(&mut self, felt: Felt) -> Felt {
        // Perform a random mutation
        let mutation_type = self.rng.gen_range(0..3);
        match mutation_type {
            0 => self.add_small_random_value(felt),
            1 => self.subtract_small_random_value(felt),
            2 => self.flip_random_bit(felt),
            // Fallback to the original value if something goes wrong
            _ => felt,
        }
    }

    fn add_small_random_value(&mut self, felt: Felt) -> Felt {
        // Random value between 1 and 9
        let small_value = self.rng.gen_range(1..10);
        felt + Felt::from(small_value)
    }

    fn subtract_small_random_value(&mut self, felt: Felt) -> Felt {
        // Random value between 1 and 9
        let small_value = self.rng.gen_range(1..10);
        felt - Felt::from(small_value)
    }

    fn flip_random_bit(&mut self, felt: Felt) -> Felt {
        // Random bit index between 0 and 251
        let bit_index = self.rng.gen_range(0..252);
        let mut felt_bytes = felt.to_bytes_be();
        felt_bytes[bit_index / 8] ^= 1 << (bit_index % 8);
        Felt::from_bytes_be(&felt_bytes)
    }
}
