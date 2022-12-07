use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};


mod fuzzer;
mod custom_rand;
mod json;
mod cairo_vm;
// JSON parsing
use fuzzer::stats::*;
use fuzzer::worker::worker;

const MAX_THREADS: u32 = 3;
fn main() {

    
    // Global statistics
    let stats = Arc::new(Mutex::new(Statistics::default()));

    // Open a log file
    let mut log = File::create("fuzz_stats.txt").unwrap();

    // Save the current time
    let start_time = Instant::now();

    for i in 0..MAX_THREADS {
        // Spawn threads
        let stats = stats.clone();
        let _ = std::thread::spawn(move || {
            worker(stats, i);
        });
    }

    loop {
        std::thread::sleep(Duration::from_millis(1000));

        // Get access to the global stats
        let stats = stats.lock().unwrap();

        let uptime = (Instant::now() - start_time).as_secs_f64();
        let fuzz_case = stats.fuzz_cases;
        print!("{:12.2} uptime | {:7} fuzz cases | {} fcps | \
                {:8} coverage | {:5} inputs | {:6} crashes [{:6} unique]\n",
            uptime, fuzz_case,
            fuzz_case as f64 / uptime,
            stats.coverage_db.len(), stats.input_db.len(),
            stats.crashes, stats.crash_db.len());

        write!(log, "{:12.0} {:7} {:8} {:5} {:6} {:6}\n",
            uptime, fuzz_case, stats.coverage_db.len(), stats.input_db.len(),
            stats.crashes, stats.crash_db.len()).unwrap();
        log.flush().unwrap();
    }
}
