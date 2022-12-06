use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::{fs, path::PathBuf};
use std::sync::{Arc, Mutex};
use std::cell::Cell;
use std::time::{Duration, Instant};
use std::path::Path;
use std::process::Command;
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// mutation for fuzzing
use basic_mutator::Mutator;
use basic_mutator::EmptyDatabase;

mod cairo_vm;
mod utils;

// JSON parsing
use utils::parse_json::parse_json;
// Execution engine
use cairo_vm::cairo_runner::runner;

const MAX_THREADS: u32 = 3;


pub fn test_target(buf: &[u8]) -> Result<Vec<(u32,u32)>, usize>{
    let mut res: Vec<(u32, u32)> = vec![];

    res.push((0,0));

    if buf.len() == 11 {
        if buf[0] as char == 'f' {
            //dprintln!("f");
            res.push((0,1));

            if buf[1] as char == 'u' {
                //dprintln!("u");
                res.push((0,2));

                if buf[2] as char == 'z' {
                    //dprintln!("z");
                    res.push((0,3));

                    if buf[3] as char == 'z' {
                        //dprintln!("z");
                        res.push((0,4));

                        if buf[4] as char == 'i' {
                            //dprintln!("i");
                            res.push((0,5));

                            if buf[5] as char == 'n' {
                                //dprintln!("n");
                                res.push((0,6));

                                if buf[6] as char == 'g' {
                                    //dprintln!("g");
                                    res.push((0,7));

                                    panic!("gg {:?}", buf);
                                }


                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(res);
}

/// A basic random number generator based on xorshift64 with 64-bits of state
struct Rng {
    /// The RNG's seed and state
    seed: u64,

    /// If set, `rand_exp` behaves the same as `rand`
    exp_disabled: bool,
}

impl Rng {
    /// Generate a random number
    #[inline]
    fn next(&mut self) -> u64 {
        let val = self.seed;
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 43;
        val
    }

    /// Generates a random number with uniform distribution in the range of
    /// [min, max]
    #[inline]
    fn rand(&mut self, min: usize, max: usize) -> usize {
        // Make sure the range is sane
        assert!(max >= min, "Bad range specified for rand()");

        // If there is no range, just return `min`
        if min == max {
            return min;
        }
        
        // If the range is unbounded, just return a random number
        if min == 0 && max == core::usize::MAX {
            return self.next() as usize;
        }

        // Pick a random number in the range
        min + (self.next() as usize % (max - min + 1))
    }
    
    /// Generates a random number with exponential distribution in the range of
    /// [min, max] with a worst case deviation from uniform of 0.5x. Meaning
    /// this will always return uniform at least half the time.
    #[inline]
    fn rand_exp(&mut self, min: usize, max: usize) -> usize {
        // If exponential random is disabled, fall back to uniform
        if self.exp_disabled {
            return self.rand(min, max);
        }

        if self.rand(0, 1) == 0 {
            // Half the time, provide uniform
            self.rand(min, max)
        } else {
            // Pick an exponentially difficult random number
            let x = self.rand(min, max);
            self.rand(min, x)
        }
    }
}

fn record_input(fuzz_input: &Vec<u8>) {
    let mut hasher = DefaultHasher::new();
    fuzz_input.hash(&mut hasher);

    let _ = std::fs::create_dir("inputs");
    std::fs::write(format!("inputs/{:016x}.input", hasher.finish()),
        format!("{:#?}", fuzz_input)).expect("Failed to save input to disk");
}

/// Sharable fuzz input
pub type FuzzInput = Arc<Vec<u8>>;

/// Fuzz case statistics
#[derive(Default)]
pub struct Statistics {
    /// Number of fuzz cases
    pub fuzz_cases: u64,

    /// Coverage database. Maps (module, offset) to `FuzzInput`s
    pub coverage_db: HashMap<Vec<(u32, u32)>, FuzzInput>,

    /// Set of all unique inputs
    pub input_db: HashSet<FuzzInput>,

    /// List of all unique inputs
    pub input_list: Vec<FuzzInput>,

    /// List of all unique inputs
    pub input_len: u64,

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
    fn get_stats_input(&self, index: usize) -> Vec<u8>{
        return self.input_list[index].to_vec();
    }
}

fn worker(stats: Arc<Mutex<Statistics>>, worker_id: u32) {
    // Local stats database
    let mut local_stats = Statistics::default();

    // TODO - make a good & clean Rng
    let seed = unsafe { core::arch::x86_64::_rdtsc() };

    // Create an RNG for this thread
    let mut rng = Rng {
                seed: seed , // 0x12640367f4b7ea35
                exp_disabled: false,
            };

    // TODO - get those info from main 
    let contract = "../cairo-libafl/tests/fuzzinglabs.json";
    let function_name = "test_symbolic_execution";
    // --contract tests/fuzzinglabs.json --function "test_symbolic_execution"
    let contents =
        fs::read_to_string(&contract.to_string()).expect("Should have been able to read the file");
    let function = match parse_json(&contents, &function_name.to_string()) {
        Some(func) => func,
        None => {
            println!("Could not find the function {}", function_name);
            return;
        }
    };

    // Create a mutator for 11-byte ASCII printable inputs
    // TODO - remove ascii limitation
    let mut mutator = Mutator::new().seed(seed)
        .max_input_size(11).printable(true);

    'next_case: loop {

        // clear previous data
        mutator.input.clear();
        // pick index 
        let index: usize = rng.rand(0, (local_stats.input_len - 1) as usize);

        if local_stats.input_len == 0 {
            // we create a first input because our db is empty
            //cov_map.new_input(&b"\0\0\0\0\0\0\0\0\0\0\0".to_vec());
            mutator.input.extend_from_slice(&b"\0\0\0\0\0\0\0\0\0\0\0".to_vec());
        } else {
            // pick from feedback corpora
            mutator.input.extend_from_slice(&local_stats.get_stats_input(index));
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
                    vec_trace.push((trace.0.offset.try_into().unwrap(), trace.1.offset.try_into().unwrap()));
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
                    local_stats.coverage_db.insert(vec_trace.clone(),
                        fuzz_input.clone());

                    // Mutex locking is limited to this scope
                    {
                        // Get access to global stats
                        let mut stats = stats.lock().unwrap();
                        if !stats.coverage_db.contains_key(&vec_trace) {
                            // Save input to global input database
                            if stats.input_db.insert(fuzz_input.clone()) {
                                stats.input_list.push(fuzz_input.clone());
                                stats.input_len +=1;
                        
                                record_input(&fuzz_input);
                            }
                            
                            // Save coverage to global coverage database
                            stats.coverage_db.insert(vec_trace.clone(), fuzz_input.clone());
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
                    stats.crashes       += 1;

                    // Add the crashing input to the input databases
                    local_stats.input_db.insert(fuzz_input.clone());
                    if stats.input_db.insert(fuzz_input.clone()) {
                        stats.input_list.push(fuzz_input.clone());
                        stats.input_len +=1;

                        record_input(&fuzz_input);

                    }

                    // TODO - generate crash name
                    let crashname: String = "crash.txt".to_string();

                    // Add the crash name and corresponding fuzz input to the crash
                    // database
                    local_stats.crash_db.insert(e.to_string(), fuzz_input.clone());
                    stats.crash_db.insert(e.to_string(), fuzz_input.clone());
                }

                println!("{:?} {:?}", &mutator.input, e);

            },
        }
        
        // TODO - only update every 1k exec to prevent lock
        let counter_update = 1000;
        if local_stats.fuzz_cases % counter_update == 1 {
            // Get access to global stats
            let mut stats = stats.lock().unwrap();
            // Update fuzz case count
            stats.fuzz_cases += counter_update;
        }
        local_stats.fuzz_cases +=1;
    }
}

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
