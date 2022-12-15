use crate::cairo_vm::cairo_types::Felt;
use crate::mutator::mutator::{EmptyDatabase, Mutator};
use std::sync::{Arc, Mutex};

use super::fuzzer::Fuzzer;
use super::inputs::{record_json_crash, record_json_input};
use super::stats::*;
use crate::cairo_vm::cairo_runner::runner;
use crate::custom_rand::rng::Rng;
use crate::{CrashCorpus, InputCorpus};

pub fn worker(
    inputs_corpus: Arc<Mutex<InputCorpus>>,
    crashes_corpus: Arc<Mutex<CrashCorpus>>,
    worker_id: i32,
    fuzzing_data: Arc<Fuzzer>,
) {
    // Local stats database
    let stats = &fuzzing_data.stats;
    let mut local_stats = Statistics::default();
    let contents = &fuzzing_data.contract_content;
    let function = &fuzzing_data.function;
    // Create an RNG for this thread, seed is unique per thread
    // to prevent duplication of efforts
    let rng = Rng::seeded(fuzzing_data.seed + (worker_id as u64)); // 0x12640367f4b7ea35

    // Create a mutator
    let mut mutator = Mutator::new()
        .seed(fuzzing_data.seed + (worker_id as u64))
        .max_input_size(function.num_args as usize);
    // TODO - IMPORTANT - Should we replay all the corpus before starting to mutate ? because we will not trigger the bug directly after running
    'next_case: loop {
        // clear previous data
        mutator.input.clear();
        if local_stats.input_db.len() > 0 {

            let index: usize = rng.rand_usize() % local_stats.input_len;
            // pick from feedback corpora
            mutator
                .input
                .extend_from_slice(&local_stats.get_stats_input(index));
        } else {
            mutator
                .input
                .extend_from_slice(&vec![b'\0' as Felt; function.num_args as usize]);
        }

        // Corrupt it with 4 mutation passes
        mutator.mutate(4, &EmptyDatabase);

        // not the good size, drop this input
        if mutator.input.len() != function.num_args as usize {
            println!(
                "Corrupted input size {} != {}",
                mutator.input.len(),
                function.num_args
            );
            continue 'next_case;
        }

        // Wrap up the fuzz input in an `Arc`
        let fuzz_input = Arc::new(mutator.input.clone());

        match runner(&contents, &function.name, &mutator.input) {
            Ok(traces) => {
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
                    if local_stats.input_db.len() != stats.input_db.len() {
                        local_stats.coverage_db = stats.coverage_db.clone();
                        local_stats.input_list = stats.input_list.clone();
                        local_stats.input_len = stats.input_len;
                        local_stats.input_db = stats.input_db.clone();
                        local_stats.crash_db = stats.crash_db.clone();
                        local_stats.crashes = stats.crashes;
                    }
                }

                // Check if this coverage entry is something we've never seen before
                if !local_stats.coverage_db.contains_key(&vec_trace) {
                    // Coverage entry is new, save the fuzz input in the input database
                    local_stats.input_db.insert(fuzz_input.clone());

                    // Update the module+offset in the coverage database to reflect that this input caused this coverage to occur
                    local_stats
                        .coverage_db
                        .insert(vec_trace.clone(), fuzz_input.clone());

                    // Mutex locking is limited to this scope
                    {
                        // Get access to global stats
                        let mut stats = stats.lock().unwrap();
                        let mut inputs_corpus = inputs_corpus.lock().unwrap();

                        if !stats.coverage_db.contains_key(&vec_trace) {
                            // Save input to global input database
                            if stats.input_db.insert(fuzz_input.clone()) {
                                inputs_corpus.inputs.push(mutator.input.clone());
                                stats.input_list.push(fuzz_input.clone());
                                stats.input_len += 1;
                                record_json_input(&inputs_corpus);
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
                    let mut crashes_corpus = crashes_corpus.lock().unwrap();
                    local_stats.crashes += 1;
                    stats.crashes += 1;
                    // Check if this case ended due to a crash
                    // Add the crashing input to the input databases
                    local_stats.input_db.insert(fuzz_input.clone());
                    if !stats.input_db.insert(fuzz_input.clone()) {
                        stats.input_list.push(fuzz_input.clone());
                        stats.input_len += 1;
                    }
                    // Add the crash name and corresponding fuzz input to the crash database
                    local_stats.crash_db.insert(fuzz_input.clone());
                    if stats.crash_db.insert(fuzz_input.clone()) {
                        // add input to the crash corpus
                        crashes_corpus.crashes.push(mutator.input.clone());
                        record_json_crash(&crashes_corpus);
                        println!(
                            "WORKER {} -- INPUT => {:?} -- ERROR \"{:?}\"",
                            worker_id, &mutator.input, e
                        );
                    }
                }
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
