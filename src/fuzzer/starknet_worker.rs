use crate::{
    fuzzer::stats::*, json::json_parser::Function, starknet_helper::starknet::StarknetFuzzer,
    starknet_helper::starknet_runner::starknet_runner,
};
use rand::{prelude::SliceRandom, SeedableRng, rngs::StdRng, Rng};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct StarknetWorker {
    stats: Arc<Mutex<Statistics>>,
    worker_id: i32,
    starknet_fuzzer: StarknetFuzzer,
    all_functions: Vec<Function>,
    seed: u64,
}
impl StarknetWorker {
    pub fn new(
        stats: Arc<Mutex<Statistics>>,
        worker_id: i32,
        starknet_fuzzer: StarknetFuzzer,
        all_functions: Vec<Function>,
        seed: u64,
    ) -> Self {
        StarknetWorker {
            stats,
            worker_id,
            starknet_fuzzer,
            all_functions,
            seed: seed,
        }
    }

    pub fn fuzz(self) {
        // Local stats database
        let mut local_stats = Statistics::default();
        let mut rng: StdRng = SeedableRng::seed_from_u64(self.seed);

        // TODO - IMPORTANT - Should we replay all the corpus before starting to mutate ? because we will not trigger the bug directly after running
        loop {
            // clear previous data

            // Wrap up the fuzz input in an `Arc`
            let n1: u8 = rng.gen();
            let mut tx_sequence: Vec<Function> = Vec::new();
            for _i in 0..n1 {
                tx_sequence.push(self.all_functions.choose(&mut rng).unwrap().clone());
            }
            // run the cairo vm
            match starknet_runner(self.stats.clone(), &tx_sequence, &self.starknet_fuzzer) {
                Ok(_) => {
                    // Mutex locking is limited to this scope
                    {
                        let stats = self.stats.lock().expect("Failed to get mutex");
                        // verify if new input has been found by other fuzzers
                        // if so, update our statistics
                        if local_stats.input_len != stats.input_len {
                            local_stats.coverage_db = stats.coverage_db.clone();
                            local_stats.input_len = stats.input_len;
                            local_stats.input_db = stats.input_db.clone();
                            local_stats.input_list = stats.input_list.clone();
                            local_stats.crash_db = stats.crash_db.clone();
                        }
                    }
                }
                Err(e) => {
                    // Mutex locking is limited to this scope
                    {
                        // Get access to global stats
                        let mut stats = self.stats.lock().expect("Failed to get mutex");

                        // Update crash counters
                        local_stats.crashes += 1;
                        stats.crashes += 1;

                        println!("WORKER {} --  -- ERROR \"{:?}\"", self.worker_id, e);
                    }
                }
            }
        }
    }
}
