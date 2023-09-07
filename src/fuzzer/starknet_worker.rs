use crate::mutator::mutator_felt252::{EmptyDatabase, Mutator};
use crate::runner::runner::Runner;
use felt::Felt252;
use starknet_rs::CasmContractClass;
use std::sync::{Arc, Mutex};

use super::stats::*;
use super::{corpus_crash::CrashFile, corpus_input::InputFile};

use crate::custom_rand::rng::Rng;
use crate::fuzzer::utils::hash_vector;
use crate::json::json_parser::Function;
use crate::runner::starknet_runner::RunnerStarknet;

pub struct StarknetWorker {
    stats: Arc<Mutex<Statistics>>,
    worker_id: i32,
    contract_class: CasmContractClass,
    function: Function,
    seed: u64,
    input_file: Arc<Mutex<InputFile>>,
    crash_file: Arc<Mutex<CrashFile>>,
    iter: i64,
}

impl StarknetWorker {
    pub fn new(
        stats: Arc<Mutex<Statistics>>,
        worker_id: i32,
        contract_class: CasmContractClass,
        function: Function,
        seed: u64,
        input_file: Arc<Mutex<InputFile>>,
        crash_file: Arc<Mutex<CrashFile>>,
        iter: i64,
    ) -> Self {
        StarknetWorker {
            stats,
            worker_id,
            contract_class,
            function,
            seed: seed,
            input_file,
            crash_file,
            iter,
        }
    }

    pub fn fuzz(self) {
        // Local stats database
        let mut local_stats = Statistics::default();

        // Create an RNG for this thread, seed is unique per thread
        // to prevent duplication of efforts
        let rng = Rng::seeded(self.seed);
        let inputs_len = self.function.inputs.len();
        // Create a mutator
        let mut mutator = Mutator::new()
            .seed(self.seed)
            .max_input_size(inputs_len)
            .types(self.function.inputs);
        let mut starknet_runner =
            RunnerStarknet::new(&self.contract_class, self.function.selector_idx);
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
                    .extend_from_slice(&vec![Felt252::from(b'\0'); inputs_len]);
            }

            // Corrupt it with 4 mutation passes
            mutator.mutate(4, &EmptyDatabase);

            // not the good size, drop this input // Useless verification
            if mutator.input.len() != inputs_len {
                println!(
                    "Corrupted input size {} != {}",
                    mutator.input.len(),
                    inputs_len
                );
                continue 'next_case;
            }

            // run the cairo vm
            match starknet_runner.clone().runner(&mutator.input) {
                Ok(res) => {
                    starknet_runner = res.0;
                    let call_info = res.1;
                    let vec_trace = call_info.trace;
                    let hash_vec = hash_vector(&vec_trace);
                    let failure_flag = call_info.failure_flag;

                    // Mutex locking is limited to this scope
                    {
                        let mut stats = self.stats.lock().expect("Failed to get mutex");
                        // failure_flag is set if the smart contract raise an error or if for example the function could not be called because of an error in the arguments
                        // TODO: to be more accurate in the args gen, we should generate the same type as in the prototype
                        if failure_flag {
                            // Update crash counters
                            local_stats.crashes += 1;
                            stats.crashes += 1;
                            // Add the crash input to the local crash database
                            local_stats.crash_db.insert(mutator.input.clone());

                            // Add the crash input to the shared crash database
                            if stats.crash_db.insert(mutator.input.clone()) {
                                // add input to the crash corpus
                                // New crashing input, we dump the crash on the disk
                                let mut crash_file_lock =
                                    self.crash_file.lock().expect("Failed to get mutex");
                                crash_file_lock.crashes.push(mutator.input.to_vec());
                                crash_file_lock.dump_json();
                            }
                        }
                        if self.iter > 0 && self.iter < stats.fuzz_cases as i64 {
                            return;
                        }
                        // verify if new input has been found by other fuzzers
                        // if so, update our statistics
                        if local_stats.input_len != stats.input_len {
                            local_stats.coverage_db = stats.coverage_db.clone();
                            local_stats.input_len = stats.input_len;
                            local_stats.input_db = stats.input_db.clone();
                            //local_stats.input_list = stats.input_list.clone();
                            local_stats.crash_db = stats.crash_db.clone();
                            local_stats.tx_crash_db = stats.tx_crash_db.clone();
                        }
                    }

                    // Mutex locking is limited to this scope
                    {
                        // Check if this coverage entry is something we've never seen before
                        if !local_stats.coverage_db.contains_key(&hash_vec) {
                            // Coverage entry is new, save the fuzz input in the input database
                            local_stats.input_db.insert(mutator.input.clone());

                            // Update the module+offset in the coverage database to reflect that this input caused this coverage to occur
                            local_stats
                                .coverage_db
                                .insert(hash_vec, mutator.input.clone());

                            // Get access to global stats
                            let mut stats = self.stats.lock().expect("Failed to get mutex");

                            if !stats.coverage_db.contains_key(&hash_vec) {
                                // Save input to global input database
                                if stats.input_db.insert(mutator.input.clone()) {
                                    // Copy in the input list
                                    //stats.input_list.push(mutator.input.clone()); Useless since we have the unique hashset input_db
                                    stats.input_len += 1;
                                    // Copy locally
                                    let mut input_file_lock =
                                        self.input_file.lock().expect("Failed to get mutex");
                                    input_file_lock.inputs.push(mutator.input.clone());
                                    input_file_lock.dump_json();
                                }
                                // Save coverage to global coverage database
                                stats.coverage_db.insert(hash_vec, mutator.input.clone());
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
                        local_stats.tx_crashes += 1;
                        stats.tx_crashes += 1;

                        // Add the crash input to the local crash database
                        local_stats.tx_crash_db.insert(mutator.input.clone());

                        // Add the crash input to the shared crash database
                        if stats.tx_crash_db.insert(mutator.input.clone()) {
                            // add input to the crash corpus
                            // New crashing input, we dump the crash on the disk
                            let mut crash_file_lock =
                                self.crash_file.lock().expect("Failed to get mutex");
                            crash_file_lock.crashes.push(mutator.input.clone());
                            crash_file_lock.dump_json();

                            println!(
                                "WORKER {} -- INPUT => {:?} -- ERROR \"{:?}\"",
                                self.worker_id, &mutator.input, e
                            );
                        }
                    }
                }
            }
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

    //TODO : Fix to match the new runner
    pub fn replay(&mut self, inputs: Vec<Vec<Felt252>>) {
        // Local stats database
        let mut local_stats = Statistics::default();
        let mut starknet_runner =
            RunnerStarknet::new(&self.contract_class, self.function.selector_idx);
        for input in inputs {
            let fuzz_input = input;
            match starknet_runner.clone().runner(&fuzz_input) {
                Ok(res) => {
                    starknet_runner = res.0;
                    let call_info = res.1;
                    let vec_trace = call_info.trace;
                    let hash_vec = hash_vector(&vec_trace);
                    let failure_flag = call_info.failure_flag;

                    // Mutex locking is limited to this scope
                    {
                        let mut stats = self.stats.lock().expect("Failed to get mutex");
                        if failure_flag {
                            // Update crash counters
                            local_stats.crashes += 1;
                            stats.crashes += 1;
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
                            }
                        }
                        // verify if new input has been found by other fuzzers
                        // if so, update our statistics
                        if local_stats.input_db.len() != stats.input_db.len() {
                            local_stats.coverage_db = stats.coverage_db.clone();
                            local_stats.input_db = stats.input_db.clone();
                            local_stats.crash_db = stats.crash_db.clone();
                        }
                    }
                    // Check if this coverage entry is something we've never seen before
                    if !local_stats.coverage_db.contains_key(&hash_vec) {
                        // Coverage entry is new, save the fuzz input in the input database
                        local_stats.input_db.insert(fuzz_input.clone());

                        // Update the module+offset in the coverage database to reflect that this input caused this coverage to occur
                        local_stats.coverage_db.insert(hash_vec, fuzz_input.clone());

                        // Mutex locking is limited to this scope
                        {
                            // Get access to global stats
                            let mut stats = self.stats.lock().expect("Failed to get mutex");

                            if !stats.coverage_db.contains_key(&hash_vec) {
                                // Save input to global input database
                                if stats.input_db.insert(fuzz_input.clone()) {
                                    //stats.input_list.push(fuzz_input.clone());
                                    stats.input_len += 1;
                                }
                                // Save coverage to global coverage database
                                stats.coverage_db.insert(hash_vec, fuzz_input.clone());
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
                        local_stats.tx_crashes += 1;
                        stats.tx_crashes += 1;

                        // Add the crash input to the local crash database
                        local_stats.tx_crash_db.insert(fuzz_input.clone());

                        // Add the crash input to the shared crash database
                        if stats.tx_crash_db.insert(fuzz_input.clone()) {
                            // add input to the crash corpus
                            // New crashing input, we dump the crash on the disk
                            let mut crash_file_lock =
                                self.crash_file.lock().expect("Failed to get mutex");
                            crash_file_lock.crashes.push(fuzz_input.to_vec());
                            crash_file_lock.dump_json();

                            println!(
                                "WORKER {} -- INPUT => {:?} -- ERROR \"{:?}\"",
                                self.worker_id, fuzz_input, e
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
