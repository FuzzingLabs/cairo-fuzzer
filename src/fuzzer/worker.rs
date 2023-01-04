use crate::cairo_vm::cairo_types::Felt;
use crate::mutator::mutator::{EmptyDatabase, Mutator};
use std::sync::{Arc, Mutex};

use super::corpus::{CrashFile, InputFile};
use super::stats::*;

use crate::cairo_vm::cairo_runner::{py_runner, runner};
use crate::custom_rand::rng::Rng;
use crate::json::json_parser::Function;

use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum WorkerError {
    // TODO implem
}

pub struct Worker {
    stats: Arc<Mutex<Statistics>>,
    worker_id: i32,
    contents: String,
    function: Function,
    seed: u64,
    input_file: Arc<Mutex<InputFile>>,
    crash_file: Arc<Mutex<CrashFile>>,
}

impl Worker {
    pub fn new(
        stats: Arc<Mutex<Statistics>>,
        worker_id: i32,
        contents: String,
        function: Function,
        seed: u64,
        input_file: Arc<Mutex<InputFile>>,
        crash_file: Arc<Mutex<CrashFile>>,
    ) -> Self {
        Worker {
            stats,
            worker_id,
            contents,
            function,
            seed: seed,
            input_file,
            crash_file,
        }
    }

    pub fn fuzz(self) {
        // Local stats database
        let mut local_stats = Statistics::default();

        // Create an RNG for this thread, seed is unique per thread
        // to prevent duplication of efforts
        let rng = Rng::seeded(self.seed);

        // Create a mutator
        let mut mutator = Mutator::new()
            .seed(self.seed)
            .max_input_size(self.function.num_args as usize);

        // TODO - IMPORTANT - Should we replay all the corpus before starting to mutate ? because we will not trigger the bug directly after running
        'next_case: loop {
            // clear previous data
            mutator.input.clear();
            if local_stats.input_len > 0 {
                let index: usize = rng.rand_usize() % local_stats.input_len;
                // pick from feedback corpora
                mutator
                    .input
                    .extend_from_slice(&local_stats.get_input_by_index(index));
            } else {
                mutator
                    .input
                    .extend_from_slice(&vec![b'\0' as Felt; self.function.num_args as usize]);
            }

            // Corrupt it with 4 mutation passes
            mutator.mutate(4, &EmptyDatabase);

            // not the good size, drop this input
            if mutator.input.len() != self.function.num_args as usize {
                println!(
                    "Corrupted input size {} != {}",
                    mutator.input.len(),
                    self.function.num_args
                );
                continue 'next_case;
            }

            // Wrap up the fuzz input in an `Arc`
            let fuzz_input = Arc::new(mutator.input.clone());

            // run the cairo vm
            match if !self.function.hints && !self.function.starknet {
                runner(&self.contents, &self.function.name, &mutator.input)
            } else {
                py_runner(
                    &self.contents,
                    &self.function.name,
                    &self.function.entrypoint,
                    &mutator.input,
                )
            } {
                Ok(traces) => {
                    let mut vec_trace: Vec<(u32, u32)> = vec![];
                    for trace in traces.expect("Failed to get traces") {
                        vec_trace.push((
                            trace
                                .0
                                .offset
                                .try_into()
                                .expect("Failed to transform offset into u32"),
                            trace
                                .1
                                .offset
                                .try_into()
                                .expect("Failed to transform offset into u32"),
                        ));
                    }
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

                    // Mutex locking is limited to this scope
                    {
                        // Check if this coverage entry is something we've never seen before
                        if !local_stats.coverage_db.contains_key(&vec_trace) {
                            // Coverage entry is new, save the fuzz input in the input database
                            local_stats.input_db.insert(fuzz_input.clone());

                            // Update the module+offset in the coverage database to reflect that this input caused this coverage to occur
                            local_stats
                                .coverage_db
                                .insert(vec_trace.clone(), fuzz_input.clone());

                            // Get access to global stats
                            let mut stats = self.stats.lock().expect("Failed to get mutex");

                            if !stats.coverage_db.contains_key(&vec_trace) {
                                // Save input to global input database
                                if stats.input_db.insert(fuzz_input.clone()) {
                                    // Copy in the input list
                                    stats.input_list.push(fuzz_input.clone());
                                    stats.input_len += 1;
                                    // Copy locally
                                    let mut input_file_lock =
                                        self.input_file.lock().expect("Failed to get mutex");
                                    input_file_lock.inputs.push(fuzz_input.to_vec());
                                    input_file_lock.dump_json();
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
                        let mut stats = self.stats.lock().expect("Failed to get mutex");

                        // Update crash counters
                        local_stats.crashes += 1;
                        stats.crashes += 1;

                        // Check if this case ended due to a crash
                        // Add the crashing input to the input databases
                        local_stats.input_db.insert(fuzz_input.clone());
                        if stats.input_db.insert(fuzz_input.clone()) {
                            stats.input_list.push(fuzz_input.clone());
                            stats.input_len += 1;
                        }
                        // Add the crash input to the local crash database
                        local_stats.crash_db.insert(fuzz_input.clone());

                        // Add the crash input to the shared crash database
                        if stats.crash_db.insert(fuzz_input.clone()) {
                            // add input to the crash corpus
                            // New crashing input, we dump the crash on the disk
                            let mut crash_file_lock =
                                self.crash_file.lock().expect("Failed to get mutex");
                            crash_file_lock.crashes.push(fuzz_input.to_vec());
                            crash_file_lock.dump_json();

                            println!(
                                "WORKER {} -- INPUT => {:?} -- ERROR \"{:?}\"",
                                self.worker_id, &mutator.input, e
                            );
                        }
                    }
                }
            }

            // TODO - only update every 1k exec to prevent lock
            let counter_update = 1000;
            if local_stats.fuzz_cases % counter_update == 1 {
                // Get access to global stats
                let mut stats = self.stats.lock().expect("Failed to get mutex");
                // Update fuzz case count
                stats.fuzz_cases += counter_update;
            }
            local_stats.fuzz_cases += 1;
        }
    }

    pub fn replay(&mut self, inputs: Vec<Arc<Vec<Felt>>>) {
        // Local stats database
        let mut local_stats = Statistics::default();

        for input in inputs {
            let fuzz_input = input.clone();
            match runner(&self.contents, &self.function.name, &input.clone()) {
                Ok(traces) => {
                    let mut vec_trace: Vec<(u32, u32)> = vec![];
                    for trace in traces.expect("Failed to get traces") {
                        vec_trace.push((
                            trace
                                .0
                                .offset
                                .try_into()
                                .expect("Failed to transform offset into u32"),
                            trace
                                .1
                                .offset
                                .try_into()
                                .expect("Failed to transform offset into u32"),
                        ));
                    }
                    // Mutex locking is limited to this scope
                    {
                        let stats = self.stats.lock().expect("Failed to get mutex");
                        // verify if new input has been found by other fuzzers
                        // if so, update our statistics
                        if local_stats.input_db.len() != stats.input_db.len() {
                            local_stats.coverage_db = stats.coverage_db.clone();
                            local_stats.input_db = stats.input_db.clone();
                            local_stats.crash_db = stats.crash_db.clone();
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
                            let mut stats = self.stats.lock().expect("Failed to get mutex");

                            if !stats.coverage_db.contains_key(&vec_trace) {
                                // Save input to global input database
                                if stats.input_db.insert(fuzz_input.clone()) {
                                    stats.input_list.push(fuzz_input.clone());
                                    stats.input_len += 1;
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
                        let mut stats = self.stats.lock().expect("Failed to get mutex");
                        local_stats.crashes += 1;
                        stats.crashes += 1;
                        // Check if this case ended due to a crash
                        // Add the crashing input to the input databases
                        local_stats.input_db.insert(fuzz_input.clone());
                        if stats.input_db.insert(fuzz_input.clone()) {
                            stats.input_list.push(fuzz_input.clone());
                            stats.input_len += 1;
                        }

                        // Add the crash name and corresponding fuzz input to the crash database
                        local_stats.crash_db.insert(fuzz_input.clone());
                        if stats.crash_db.insert(fuzz_input.clone()) {
                            // add input to the crash corpus
                            println!(
                                "WORKER {} -- INPUT => {:?} -- ERROR \"{:?}\"",
                                self.worker_id, &input, e
                            );
                        }
                    }
                }
            }

            // Get access to global stats
            let mut stats = self.stats.lock().expect("Failed to get mutex");
            // Update fuzz case count
            stats.fuzz_cases += 1;
            local_stats.fuzz_cases += 1;
        }

        // Update the threads_finished when the worker executes all the corpus chunk
        let mut stats = self.stats.lock().expect("Failed to get mutex");
        stats.threads_finished += 1;
    }
}
