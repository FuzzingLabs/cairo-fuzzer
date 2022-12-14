use std::fs::File;
/// Sharable fuzz input
use std::sync::Arc;
pub type FuzzInput = Arc<Vec<Felt>>;
use std::collections::{HashMap};
use std::io::Write;
use std::time::{Duration, Instant};

use crate::cairo_vm::cairo_types::Felt;
use crate::FuzzingData;

/// Fuzz case statistics
#[derive(Default, Debug)]
pub struct Statistics {
    /// Number of fuzz cases
    pub fuzz_cases: u64,

    /// Coverage database. Maps (module, offset) to `FuzzInput`s
    pub coverage_db: HashMap<Vec<(u32, u32)>, FuzzInput>,

    /// Set of all unique inputs
    pub input_db: Vec<FuzzInput>,

    /// List of all unique crashes
    pub crash_list: HashMap<String, Felt>,

    /// Unique set of fuzzer actions
    ///pub unique_action_set: HashSet<FuzzerAction>,

    /// List of all unique fuzzer actions
    ///pub unique_actions: Vec<FuzzerAction>,

    /// Database of crash file names to `FuzzInput`s
    pub crashes: u64,
    pub crash_db: HashMap<String, FuzzInput>,

    /// Set number of threads that stopped running
    pub finished: u64,

    /// Removed files
    pub removed_files: u64,
}


pub fn print_stats(fuzzing_data: Arc<FuzzingData>, replay:bool, workers: usize) {
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
            stats.input_db.len(),
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
                stats.input_db.len(),
                stats.crashes,
                stats.crash_db.len()
            )
            .unwrap();
            file.flush().unwrap();
        }
        if replay && stats.finished == workers as u64 {
            break;
        }
    }
}
