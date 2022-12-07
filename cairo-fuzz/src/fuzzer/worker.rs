use basic_mutator::EmptyDatabase;
use basic_mutator::Mutator;
use std::sync::{Arc, Mutex};

use super::inputs::record_input;
use super::stats::*;
use crate::cairo_vm::cairo_runner::runner;
use crate::custom_rand::rng::Rng;
use crate::FuzzingData;

pub fn worker(stats: Arc<Mutex<Statistics>>, worker_id: i32, fuzzing_data: Arc<FuzzingData>) {
    // Local stats database
    let mut local_stats = Statistics::default();
    let contents = &fuzzing_data.contents;
    let function = &fuzzing_data.function;
    let seed = fuzzing_data.seed;
    // Create an RNG for this thread
    let rng = Rng::seeded(seed); // 0x12640367f4b7ea35

    // Create a mutator for 11-byte ASCII printable inputs
    // TODO - remove ascii limitation
    let mut mutator = Mutator::new().seed(seed).max_input_size(11).printable(true);

    'next_case: loop {
        // clear previous data
        mutator.input.clear();
        // pick index

        let index: usize = if local_stats.input_len > 0 {
            rng.rand_usize() % local_stats.input_len
        } else {
            0
        };

        if local_stats.input_len == 0 {
            // we create a first input because our db is empty
            //cov_map.new_input(&b"\0\0\0\0\0\0\0\0\0\0\0".to_vec());
            mutator
                .input
                .extend_from_slice(&b"\0\0\0\0\0\0\0\0\0\0\0".to_vec());
        } else {
            // pick from feedback corpora
            mutator
                .input
                .extend_from_slice(&local_stats.get_stats_input(index));
        }

        // Corrupt it with 4 mutation passes
        mutator.mutate(4, &EmptyDatabase);

        // not the good size, drop this input
        // TODO - remove mutator that change the input size
        if mutator.input.len() != 11 {
            continue 'next_case;
        }

        // Wrap up the fuzz input in an `Arc`
        let fuzz_input = Arc::new(mutator.input.clone());

        match runner(&contents, function.name.clone(), &mutator.input) {
            Ok(traces) => {
                //println!("traces = {:?}", traces);
                let mut vec_trace: Vec<(u32, u32)> = vec![];
                for trace in traces.unwrap() {
                    vec_trace.push((
                        trace.0.offset.try_into().unwrap(),
                        trace.1.offset.try_into().unwrap(),
                    ));
                }

                // Mutex locking is limited to this scope
                {
                    let stats = stats.lock().unwrap();
                    // verify if new input has been found by other fuzzers
                    // if so, update our statistics
                    if local_stats.input_len != stats.input_len {
                        local_stats.coverage_db = stats.coverage_db.clone();
                        local_stats.input_db = stats.input_db.clone();
                        local_stats.input_list = stats.input_list.clone();
                        local_stats.input_len = stats.input_len;
                        local_stats.crashes = stats.crashes;
                        local_stats.crash_db = stats.crash_db.clone();
                    }
                }

                // Check if this coverage entry is something we've never seen before
                if !local_stats.coverage_db.contains_key(&vec_trace) {
                    // Coverage entry is new, save the fuzz input in the input
                    // database
                    local_stats.input_db.insert(fuzz_input.clone());

                    // Update the module+offset in the coverage database to
                    // reflect that this input caused this coverage to occur
                    local_stats
                        .coverage_db
                        .insert(vec_trace.clone(), fuzz_input.clone());

                    // Mutex locking is limited to this scope
                    {
                        // Get access to global stats
                        let mut stats = stats.lock().unwrap();
                        if !stats.coverage_db.contains_key(&vec_trace) {
                            // Save input to global input database
                            if stats.input_db.insert(fuzz_input.clone()) {
                                stats.input_list.push(fuzz_input.clone());
                                stats.input_len += 1;

                                record_input(&fuzz_input, false);
                            }

                            // Save coverage to global coverage database
                            stats
                                .coverage_db
                                .insert(vec_trace.clone(), fuzz_input.clone());
                        }
                    }
                }
            }
            Err(e) => {
                // Mutex locking is limited to this scope
                {
                    // Get access to global stats
                    let mut stats = stats.lock().unwrap();

                    // Check if this case ended due to a crash
                    // Update crash information
                    local_stats.crashes += 1;
                    stats.crashes += 1;

                    // Add the crashing input to the input databases
                    local_stats.input_db.insert(fuzz_input.clone());
                    if stats.input_db.insert(fuzz_input.clone()) {
                        stats.input_list.push(fuzz_input.clone());
                        stats.input_len += 1;

                        record_input(&fuzz_input, true);
                    }

                    // Add the crash name and corresponding fuzz input to the crash
                    // database
                    local_stats
                        .crash_db
                        .insert(e.to_string(), fuzz_input.clone());
                    stats.crash_db.insert(e.to_string(), fuzz_input.clone());
                }

                println!(
                    "WORKER {} -- INPUT => {:?} -- ERROR \"{:?}\"",
                    worker_id, &mutator.input, e
                );
            }
        }

        // TODO - only update every 1k exec to prevent lock
        let counter_update = 1000;
        if local_stats.fuzz_cases % counter_update == 1 {
            // Get access to global stats
            let mut stats = stats.lock().unwrap();
            // Update fuzz case count
            stats.fuzz_cases += counter_update;
        }
        local_stats.fuzz_cases += 1;
    }
}
