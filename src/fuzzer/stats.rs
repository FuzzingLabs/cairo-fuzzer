use felt::Felt252;
use std::collections::{HashMap, HashSet};
pub type FuzzInput = Vec<Felt252>;

/// Fuzz case statistics
#[derive(Default, Debug)]
pub struct Statistics {
    /// Number of fuzz cases
    pub fuzz_cases: u64,

    /// Coverage database. Maps (module, offset) to `FuzzInput`s
    pub coverage_db: HashMap<u64, FuzzInput>,

    /// Counter of inputs
    pub input_len: usize,

    /// Set of all unique inputs
    pub input_db: HashSet<FuzzInput>,

    /// List of inputs
    /* pub input_list: Vec<FuzzInput>, */

    /// List of all unique fuzzer actions

    /// Counter of crashes
    pub crashes: u64,

    /// Set of all unique crashes
    pub crash_db: HashSet<FuzzInput>,

    /// Contains the hash of the trace vector to verify if the crash is unique or not
    pub crash_coverage: u64,

    /// Counter of crashes
    pub tx_crashes: u64,

    /// Set of all unique crashes
    pub tx_crash_db: HashSet<FuzzInput>,

    // Number of threads that finished to run
    pub threads_finished: u64,
}

impl Statistics {
    pub fn get_input_by_index(&self, index: usize) -> &FuzzInput {
        let mut iterator = self.input_db.iter();
        iterator
            .nth(index)
            .expect("Could not get element from input_db")
    }
}
