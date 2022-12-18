use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::cairo_vm::cairo_types::Felt;

pub type FuzzInput = Arc<Vec<Felt>>;

/// Fuzz case statistics
#[derive(Default, Debug)]
pub struct Statistics {
    /// Number of fuzz cases
    pub fuzz_cases: u64,

    /// Coverage database. Maps (module, offset) to `FuzzInput`s
    pub coverage_db: HashMap<Vec<(u32, u32)>, FuzzInput>,

    /// Counter of inputs
    pub input_len: usize,

    /// Set of all unique inputs
    pub input_db: HashSet<FuzzInput>,

    /// List of inputs
    pub input_list: Vec<FuzzInput>,

    // add counter of size
    /// Unique set of fuzzer actions
    ///pub unique_action_set: HashSet<FuzzerAction>,

    /// List of all unique fuzzer actions
    ///pub unique_actions: Vec<FuzzerAction>,

    /// Counter of crashes
    pub crashes: u64,

    /// Set of all unique crashes
    pub crash_db: HashSet<FuzzInput>,

    /// Database of crash file names to `FuzzInput`s
    // pub crash_db: HashMap<String, FuzzInput>,

    // TODO Add counter of unique crashes
    pub threads_finished: u64,
}

impl Statistics {
    pub fn get_input_by_index(&self, index: usize) -> FuzzInput {
        self.input_list[index].clone()
    }
}

/* pub fn print_stats(fuzzing_data: &Arc<FuzzingData>, replay: bool, workers: usize) {
    let mut log = None;
    if fuzzing_data.logs {
        log = Some(File::create("fuzz_stats.txt").unwrap());
    }
    loop {
        std::thread::sleep(Duration::from_millis(1000));

        // Get access to the global stats
        let stats = fuzzing_data.stats.lock().unwrap();

        let uptime = (Instant::now() - fuzzing_data.start_time).as_secs_f64();
        let fuzz_case = stats.fuzz_cases;
        print!(
            "{:12.2} uptime | {:9} fuzz cases | {:12.2} fcps | \
                    {:6} coverage | {:6} inputs | {:6} crashes [{:6} unique]\n",
            uptime,
            fuzz_case,
            fuzz_case as f64 / uptime,
            stats.coverage_db.len(),
            stats.input_len,
            stats.crashes,
            stats.crash_db.len()
        );
        if let Some(ref mut file) = log {
            write!(
                file,
                "{:12.0} {:7} {:8} {:5} {:6} {:6}\n",
                uptime,
                fuzz_case,
                stats.coverage_db.len(),
                stats.input_len,
                stats.crashes,
                stats.crash_db.len()
            )
            .unwrap();
            file.flush().unwrap();
        }
        if replay && stats.threads_finished == workers as u64 {
            break;
        }
    }
} */
