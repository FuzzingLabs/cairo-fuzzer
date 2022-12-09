/// Sharable fuzz input
use std::sync::Arc;
pub type FuzzInput = Arc<Vec<u8>>;
use std::collections::{HashMap, HashSet};

/// Fuzz case statistics
#[derive(Default)]
pub struct Statistics {
    /// Number of fuzz cases
    pub fuzz_cases: u64,

    /// Coverage database. Maps (module, offset) to `FuzzInput`s
    pub coverage_db: HashMap<Vec<(u32, u32)>, FuzzInput>,
    pub coverage_minimizer_db: HashMap<Vec<(u32, u32)>, Vec<u8>>,
    /// Set of all unique inputs
    pub input_db: HashSet<FuzzInput>,
    pub input_minimizer_db: HashSet<Vec<u8>>,
    /// List of all unique crashes
    pub crash_list: HashMap<String, u128>,
    pub crash_minimizer_list: Vec<String>,
    /// List of all unique inputs
    pub input_list: Vec<FuzzInput>,
    pub input_minimizer_list: Vec<u8>,

    /// List of all unique inputs
    pub input_len: usize,

    /// Unique set of fuzzer actions
    ///pub unique_action_set: HashSet<FuzzerAction>,

    /// List of all unique fuzzer actions
    ///pub unique_actions: Vec<FuzzerAction>,

    /// Number of crashes
    pub crashes: u64,

    /// Database of crash file names to `FuzzInput`s
    pub crash_db: HashMap<String, FuzzInput>,

    /// Removed files
    pub removed_files: u64,
}
impl Statistics {
    pub fn get_stats_input(&self, index: usize) -> Vec<u8> {
        return self.input_list[index].to_vec();
    }
}
