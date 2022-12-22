use std::{
    fs::{self, File},
    process,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    cairo_vm::cairo_types::Felt,
    cli::config::Config,
    fuzzer::worker::Worker,
    json::json_parser::{parse_json, Function},
};

use super::{
    corpus::{CrashFile, InputFile},
    stats::Statistics,
};
use std::io::Write;

#[derive(Clone)]
pub struct Fuzzer {
    /// Shared fuzzing statistics between threads
    pub stats: Arc<Mutex<Statistics>>,
    /// Number of cores/threads
    pub cores: i32,
    /// Contract JSON path
    pub contract_file: String,
    /// Contract JSON content
    pub contract_content: String,
    /// Contract function to fuzz
    pub function: Function,
    /// Store local/on-disk logs
    pub logs: bool,
    /// Replay mode
    pub replay: bool,
    /// Corpus minimization
    pub minimizer: bool,
    /// Seed number
    pub seed: u64,
    /// Workspace to use
    pub workspace: String,
    /// Inputs file path
    pub input_file: Arc<Mutex<InputFile>>,
    /// Crashes file path
    pub crash_file: Arc<Mutex<CrashFile>>,
    /// Number of second the fuzzing session will last
    pub run_time: Option<u64>,
    /// Starting time of the fuzzer
    pub start_time: Instant,
    /// Running workers
    pub running_workers: u64,
}

impl Fuzzer {
    /// Create the fuzzer using the given Config struct
    pub fn new(config: &Config) -> Self {
        let stats = Arc::new(Mutex::new(Statistics::default()));
        // Set seed if provided or generate a new seed using `SystemTime`
        let seed = match config.seed {
            Some(val) => val,
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };
        println!("\t\t\t\t\t\t\tSeed: {}", seed);

        // Read contract JSON artifact and get its content
        let contents = fs::read_to_string(&config.contract_file)
            .expect("Should have been able to read the file");

        // TODO - remove when support multiple txs
        let function = match parse_json(&contents, &config.function_name) {
            Some(func) => func,
            None => {
                process::exit(1);
            }
        };

        // Load inputs from the input file if provided
        let inputs: InputFile = match config.input_file.is_empty() && config.input_folder.is_empty()
        {
            true => InputFile::new_from_function(&function, &config.workspace),
            false => match config.input_folder.is_empty() {
                true => InputFile::load_from_file(&config.input_file, &config.workspace),
                false => InputFile::load_from_folder(&config.input_folder, &config.workspace),
            },
        };
        println!("\t\t\t\t\t\t\tInputs loaded {}", inputs.inputs.len());

        // Load existing inputs in shared database
        if inputs.inputs.len() > 0 {
            let mut stats_db = stats.lock().unwrap();
            for input in inputs.inputs.clone() {
                if stats_db.input_db.insert(Arc::new(input.clone())) {
                    stats_db.input_list.push(Arc::new(input.clone()));
                    stats_db.input_len += 1;
                }
            }
        }

        // Load crashes from the crash file if provided
        let crashes: CrashFile =
            match config.crash_file.is_empty() && config.crash_folder.is_empty() {
                true => CrashFile::new_from_function(&function, &config.workspace),
                false => match config.input_folder.is_empty() {
                    true => CrashFile::load_from_file(&config.input_file, &config.workspace),
                    false => CrashFile::load_from_folder(&config.input_folder, &config.workspace),
                },
            };

        // Load existing crashes in shared database
        if crashes.crashes.len() > 0 {
            let mut stats_db = stats.lock().unwrap();
            for input in crashes.crashes.clone() {
                stats_db.crash_db.insert(Arc::new(input.clone()));
                stats_db.crashes += 1;
            }
        }

        // Setup the mutex for the inputs corpus and crash corpus
        let inputs = Arc::new(Mutex::new(inputs));
        let crashes = Arc::new(Mutex::new(crashes));

        // Setup the fuzzer
        Fuzzer {
            // Init stats struct
            stats: stats,
            cores: config.cores,
            logs: config.logs,
            run_time: config.run_time,
            replay: config.replay,
            minimizer: config.minimizer,
            contract_file: config.contract_file.clone(),
            contract_content: contents,
            function: function,
            // Init starting time
            start_time: Instant::now(),
            seed: seed,
            input_file: inputs,
            crash_file: crashes,
            workspace: config.workspace.clone(),
            running_workers: 0,
        }
    }

    /// Fuzz
    pub fn fuzz(&mut self) {
        // Running all the threads
        for i in 0..self.cores {
            // create dedicated statistics per thread
            let stats = self.stats.clone();
            let contract_content = self.contract_content.clone();
            let function = self.function.clone();
            let input_file = self.input_file.clone();
            let crash_file = self.crash_file.clone();
            let seed = self.seed + (i as u64); // create unique seed per worker

            // Spawn threads
            let _ = std::thread::spawn(move || {
                let worker = Worker::new(
                    stats,
                    i,
                    contract_content,
                    function,
                    seed,
                    input_file,
                    crash_file,
                );
                worker.fuzz();
            });
            //println!("Thread {} Spawned", i);
            self.running_workers += 1;
        }
        println!("\t\t\t\t\t\t\tRunning {} threads", self.running_workers);
        println!("        =========================================================================================================================");
        // Call the stats monitoring/printer
        self.monitor();
    }

    /// Replay a given corpus.
    /// If `minimizer` is set to "true" it will dump the new corpus
    pub fn replay(&mut self) {
        // Replay all inputs
        let stats_db = self.stats.lock().unwrap();
        // Load inputs
        let mut corpus = stats_db.input_list.clone();
        println!("Total inputs to replay => {}", corpus.len());
        // Load crashes
        let mut crashes = stats_db.crash_db.clone().into_iter().collect();
        corpus.append(&mut crashes);
        drop(stats_db);
        // Split the inputs into chunks
        let chunk_size = if corpus.len() > (self.cores as usize) {
            corpus.len() / (self.cores as usize)
        } else {
            1
        };
        let mut chunks = Vec::new();
        for chunk in corpus.chunks(chunk_size) {
            chunks.push(chunk.to_vec());
        }
        println!("Total inputs to replay => {}", corpus.len());

        let mut threads = Vec::new();

        for i in 0..chunks.len() {
            // Spawn threads
            let stats_thread = self.stats.clone();
            let contract_content = self.contract_content.clone();
            let function = self.function.clone();
            let seed = self.seed;
            let input_file = self.input_file.clone();
            let crash_file = self.crash_file.clone();

            let chunk = chunks[i].clone();
            threads.push(std::thread::spawn(move || {
                let mut worker = Worker::new(
                    stats_thread,
                    i as i32,
                    contract_content,
                    function,
                    seed,
                    input_file,
                    crash_file,
                );
                worker.replay(chunk);
            }));
            println!("Thread {} Spawned", i);
            self.running_workers += 1;
        }

        // Wait for all threads to complete
        for thread in threads {
            let _ = thread.join();
        }
        // Print stats of the current fuzzer
        self.monitor();

        // If minimizer is set, dump the new corpus
        if self.minimizer {
            let stats = self.stats.lock().unwrap();
            // Init the struct
            let mut dump_inputs = InputFile {
                workspace: self.workspace.clone(),
                path: format!("{}_min.json", self.function.name),
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
            dump_inputs.dump_json();
        }
    }

    /// Function to print stats of the running fuzzer
    fn monitor(&self) {
        let mut log = None;
        if self.logs {
            log = Some(File::create("fuzz_stats.txt").unwrap());
        }

        // Monitoring loop
        loop {
            // wait 1 second
            std::thread::sleep(Duration::from_millis(1000));

            // Get uptime
            let uptime = (Instant::now() - self.start_time).as_secs_f64();

            // Get access to the global stats
            {
                let stats = self.stats.lock().unwrap();

                // number of executions
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
                // Writing inside logging file
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

                // Only for replay: all thread are finished
                if self.replay && stats.threads_finished == self.running_workers {
                    break;
                }
            }

            // time over, fuzzing session is finished
            if let Some(run_time) = self.run_time {
                if uptime > run_time as f64 {
                    process::exit(0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{thread, time::Duration};

    use crate::cli::config::Config;

    use super::Fuzzer;
    #[test]
    fn test_loading_config_file() {
        let config_file = "tests/config.json".to_string();
        let config = Config::load_config(&config_file);
        let fuzzer = Fuzzer::new(&config);
        assert_eq!(fuzzer.cores, 1);
        assert_eq!(fuzzer.logs, false);
        assert_eq!(fuzzer.function.name, "test_symbolic_execution");
    }

    #[test]
    fn test_run_fuzzer_from_config_file() {
        let config_file = "tests/config.json".to_string();
        let config = Config::load_config(&config_file);
        let mut fuzzer = Fuzzer::new(&config);
        // Create a new thread
        let handle = thread::spawn(move || {
            fuzzer.run_time = Some(10);
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
    #[test]
    fn test_init_config() {
        let cores: i32 = 1;
        let logs: bool = false;
        let seed: Option<u64> = Some(1000);
        let run_time: Option<u64> = Some(10);
        let replay: bool = false;
        let minimizer: bool = false;
        let contract_file: String = "tests/fuzzinglabs.json".to_string();
        let function_name: String = "test_symbolic_execution".to_string();
        let input_file: String = "".to_string();
        let crash_file: String = "".to_string();
        let workspace: String = "fuzzer_workspace".to_string();
        let input_folder: String = "".to_string();
        let crash_folder: String = "".to_string();
        let config = Config {
            input_folder: input_folder,
            crash_folder: crash_folder,
            workspace,
            contract_file,
            function_name,
            input_file,
            crash_file,
            cores,
            logs,
            seed,
            run_time,
            replay,
            minimizer,
        };
        let fuzzer = Fuzzer::new(&config);
        assert_eq!(fuzzer.cores, 1);
        assert_eq!(fuzzer.logs, false);
        assert_eq!(fuzzer.function.name, "test_symbolic_execution");
    }

    #[test]
    fn test_run_fuzzer() {
        let cores: i32 = 1;
        let logs: bool = false;
        let seed: Option<u64> = Some(1000);
        let run_time: Option<u64> = Some(10);
        let replay: bool = false;
        let minimizer: bool = false;
        let contract_file: String = "tests/fuzzinglabs.json".to_string();
        let function_name: String = "test_symbolic_execution".to_string();
        let input_file: String = "".to_string();
        let crash_file: String = "".to_string();
        let workspace: String = "fuzzer_workspace".to_string();
        let input_folder: String = "".to_string();
        let crash_folder: String = "".to_string();
        let config = Config {
            input_folder: input_folder,
            crash_folder: crash_folder,
            workspace,
            contract_file,
            function_name,
            input_file,
            crash_file,
            cores,
            logs,
            seed,
            run_time,
            replay,
            minimizer,
        };
        // create the fuzzer
        let mut fuzzer = Fuzzer::new(&config);

        // Create a new thread
        let handle = thread::spawn(move || {
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

    #[test]
    fn test_replay() {
        let cores: i32 = 3;
        let logs: bool = false;
        let seed: Option<u64> = Some(1000);
        let run_time: Option<u64> = Some(10);
        let replay: bool = true;
        let minimizer: bool = false;
        let contract_file: String = "tests/fuzzinglabs.json".to_string();
        let function_name: String = "test_symbolic_execution".to_string();
        let input_file: String = "tests/test_symbolic_execution_inputs.json".to_string();
        let crash_file: String = "".to_string();
        let workspace: String = "fuzzer_workspace".to_string();
        let input_folder: String = "".to_string();
        let crash_folder: String = "".to_string();
        let config = Config {
            input_folder: input_folder,
            crash_folder: crash_folder,
            workspace,
            contract_file,
            function_name,
            input_file,
            crash_file,
            cores,
            logs,
            seed,
            run_time,
            replay,
            minimizer,
        };
        // create the fuzzer
        let mut fuzzer = Fuzzer::new(&config);

        fuzzer.replay();

        let stats = fuzzer.stats.lock().unwrap();
        assert_ne!(stats.coverage_db.len(), 0);
    }
}
