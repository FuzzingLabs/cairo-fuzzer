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

    /// Set of all unique inputs
    pub input_db: HashSet<FuzzInput>,
 
    /// List of all unique crashes
    pub crash_list: HashMap<String, u128>,

    /// List of all unique inputs
    pub input_list: Vec<FuzzInput>,

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
}
impl Statistics {
    pub fn get_stats_input(&self, index: usize) -> Vec<u8> {
        return self.input_list[index].to_vec();
    }
}
