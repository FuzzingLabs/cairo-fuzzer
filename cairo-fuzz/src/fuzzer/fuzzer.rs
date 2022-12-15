use std::{
    fs::{self, File},
    path::Path,
    process,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

use crate::{
    cairo_vm::cairo_types::Felt,
    cli::config::Config,
    fuzzer::{fuzzing_worker::worker, inputs::record_json_input, replay_worker::replay},
    json::json_parser::{parse_json, Function},
};

use super::{
    corpus::{CrashCorpus, InputCorpus},
    stats::Statistics,
};
use std::io::Write;

#[derive(Clone)]
pub struct Fuzzer {
    pub stats: Arc<Mutex<Statistics>>,
    pub cores: i32,
    pub logs: bool,
    pub replay: bool,
    pub minimizer: bool,
    pub contract_file: String,
    pub contract_content: String,
    pub function: Function,
    pub start_time: Instant,
    pub seed: u64,
    pub input_file: String,
    pub crash_file: String,
    pub workers: u64,
    pub timeout: u64,
}

impl Fuzzer {
    pub fn fuzz(&mut self) {
        // Setup input corpus and crash corpus
        let inputs = self.load_inputs_corpus();
        {
            let mut stats_db = self.stats.lock().unwrap();
            for input in inputs.inputs.clone() {
                stats_db.input_db.insert(Arc::new(input.clone()));
                stats_db.input_list.push(Arc::new(input.clone()));
                stats_db.input_len += 1;
            }
        }
        let crashes = self.load_crashes_corpus();
        // Setup the mutex for the inputs corpus and crash corpus
        let inputs = Arc::new(Mutex::new(inputs));
        let crashes = Arc::new(Mutex::new(crashes));
        // Running all the threads
        for i in 0..self.cores {
            // Spawn threads
            let fuzzing_data = Arc::new(self.clone()).clone();
            let inputs_corpus = inputs.clone();
            let crashes_corpus = crashes.clone();
            let _ = std::thread::spawn(move || {
                worker(inputs_corpus, crashes_corpus, i, fuzzing_data);
            });
            println!("Thread {} Spawned", i);
        }
        self.workers = self.cores as u64;
        // Call the stats printer
        self.print_stats();
    }

    /// Function to replay a corpus. If `minimizer` is set to "true" it will dump the new corpus
    pub fn replay(&mut self) {
        let inputs = self.load_inputs_corpus();
        let crashes = self.load_crashes_corpus();

        // Select if the corpus should be the inputs one or the crashes one
        let corpus = if self.crash_file.clone().len() == 0 && inputs.inputs.len() != 0 {
            inputs.inputs
        } else {
            crashes.crashes
        };
        // Split the files into chunks
        let chunk_size = if corpus.len() > (self.cores as usize) {
            corpus.len() / (self.cores as usize)
        } else {
            1
        };
        let mut chunks = Vec::new();
        for chunk in corpus.chunks(chunk_size) {
            chunks.push(chunk.to_vec());
        }
        println!("Total inputs => {}", corpus.len());
        for i in 0..chunks.len() {
            // Spawn threads
            let fuzzing_data = Arc::new(self.clone()).clone();
            let chunk = chunks[i].clone();
            let _ = std::thread::spawn(move || {
                replay(i, fuzzing_data, chunk);
            });
            println!("Thread {} Spawned", i);
            self.workers += 1;
        }
        // Print stats of the current fuzzer
        self.print_stats();

        // If minimizer is set, dump the new corpus
        if self.minimizer {
            let stats = self.stats.lock().unwrap();
            // Init the struct
            let mut dump_inputs = InputCorpus {
                name: self.function.name.clone(),
                args: self.function.type_args.clone(),
                inputs: Vec::<Vec<Felt>>::new(),
            };
            // Push every input to the struct
            for input in stats.input_db.clone() {
                dump_inputs.inputs.push(input.clone().to_vec());
            }
            println!("Size after minimization : {}", dump_inputs.inputs.len());
            // Dump the struct
            record_json_input(&dump_inputs);
        }
    }

    /// Function to load the previous corpus if it exists
    fn load_inputs_corpus(&self) -> InputCorpus {
        // Init the struct
        let mut inputs_corpus = InputCorpus {
            name: self.function.name.clone(),
            args: self.function.type_args.clone(),
            inputs: Vec::<Vec<Felt>>::new(),
        };
        // Set the filename based on the function name or if a crash_file was set via config file or CLI
        let filename = if self.input_file.len() == 0 {
            format!("inputs_corpus/{}_inputs.json", self.function.name)
        } else {
            self.input_file.clone()
        };
        if Path::new(&filename).is_file() {
            let contents =
                fs::read_to_string(filename).expect("Should have been able to read the file");
            let data: Value = serde_json::from_str(&contents).expect("JSON was not well-formatted");
            // Load old inputs to prevent overwriting and to use it as a dictionary for the mutator
            let inputs: Vec<Vec<Felt>> = data["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|input_array| {
                    input_array
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|input| input.as_u64().unwrap() as Felt)
                        .collect()
                })
                .collect();
            // Add the old corpus to the struct
            inputs_corpus.inputs.extend(inputs);
        }
        return inputs_corpus;
    }

    /// Function to load a crashes corpus
    fn load_crashes_corpus(&self) -> CrashCorpus {
        // Init the struct
        let mut crashes_corpus = CrashCorpus {
            name: self.function.name.clone(),
            args: self.function.type_args.clone(),
            crashes: Vec::<Vec<Felt>>::new(),
        };
        // Set the filename based on the function name or if a crash_file was set via config file or CLI
        let filename = if self.crash_file.len() == 0 {
            format!("crashes_corpus/{}_crashes.json", self.function.name)
        } else {
            self.crash_file.clone()
        };
        if Path::new(&filename).is_file() {
            let contents =
                fs::read_to_string(filename).expect("Should have been able to read the file");
            let data: Value = serde_json::from_str(&contents).expect("JSON was not well-formatted");
            // Load old crashes to prevent overwriting and to use it as a dictionary for the mutator
            let crashes: Vec<Vec<Felt>> = data["crashes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|input_array| {
                    input_array
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|input| input.as_u64().unwrap() as Felt)
                        .collect()
                })
                .collect();
            // Add the old corpus to the struct
            crashes_corpus.crashes.extend(crashes);
        }
        return crashes_corpus;
    }

    /// Function to print stats of the running fuzzer
    fn print_stats(&self) {
        let mut log = None;
        if self.logs {
            log = Some(File::create("fuzz_stats.txt").unwrap());
        }
        loop {
            std::thread::sleep(Duration::from_millis(1000));

            // Get access to the global stats
            let stats = self.stats.lock().unwrap();

            let uptime = (Instant::now() - self.start_time).as_secs_f64();
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
            if self.replay && stats.threads_finished == self.workers as u64 {
                break;
            }
            if self.timeout != 0 && uptime > self.timeout as f64 {
                process::exit(0);
            }
        }
    }
}

/// Function that init the fuzzer using the Config struct
pub fn init_fuzzer_from_config(config: Config) -> Fuzzer {
    return init_fuzzer(
        config.cores,
        config.logs,
        config.seed,
        config.timeout,
        config.replay,
        config.minimizer,
        &config.contract_file,
        &config.function_name,
        &config.input_file,
        &config.crash_file,
    );
}

/// Function to init the fuzzer
pub fn init_fuzzer(
    cores: i32,
    logs: bool,
    seed: Option<u64>,
    timeout: Option<u64>,
    replay: bool,
    minimizer: bool,
    contract_file: &String,
    function_name: &String,
    input_file: &String,
    crash_file: &String,
) -> Fuzzer {
    // Init seed
    let start_time = Instant::now();
    let set_seed = match seed {
        Some(val) => val,
        None => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };
    let set_timeout = match timeout {
        Some(val) => val,
        None => 0,
    };
    println!("Fuzzing SEED => {}", set_seed);
    // Init stats struct
    let stats = Arc::new(Mutex::new(Statistics::default()));

    // Read json artifact and get its content
    let contents = fs::read_to_string(&contract_file.to_string())
        .expect("Should have been able to read the file");
    let function = match parse_json(&contents, &function_name.to_string()) {
        Some(func) => func,
        None => {
            process::exit(1);
        }
    };
    // Setup the fuzzer
    let fuzzer = Fuzzer {
        stats: stats,
        cores: cores,
        logs: logs,
        timeout: set_timeout,
        replay: replay,
        minimizer: minimizer,
        contract_file: contract_file.to_string(),
        contract_content: contents,
        function: function,
        start_time: start_time,
        seed: set_seed,
        input_file: input_file.to_string(),
        crash_file: crash_file.to_string(),
        workers: 0,
    };
    return fuzzer;
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{thread, time::Duration};

    use crate::cli::config::load_config;

    use super::init_fuzzer_from_config;
    #[test]
    fn test_init_fuzzer_from_config_file() {
        let config_file = "tests/config.json".to_string();
        let config = load_config(&config_file);
        let mut fuzzer = init_fuzzer_from_config(config.clone());
        // Create a new thread
        let handle = thread::spawn(move || {
            fuzzer.timeout = 10;
            fuzzer.fuzz();
        });

        thread::sleep(Duration::from_secs(5));
        if handle.is_finished() {
            panic!("Process should be running");
        }

        thread::sleep(Duration::from_secs(6));
        if !handle.is_finished() {
            panic!("Process should not be running");
        }
    }
}
