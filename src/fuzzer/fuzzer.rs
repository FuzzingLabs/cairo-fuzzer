use super::{
    corpus::{CrashFile, InputFile},
    stats::Statistics,
};
use crate::{
    cairo_vm::cairo_types::Felt,
    cli::config::Config,
    fuzzer::cairo_worker::Worker,
    fuzzer::starknet_worker::StarknetWorker,
    json::json_parser::parse_starknet_json,
    json::json_parser::{parse_json, Function},
    starknet_helper,
    starknet_helper::devnet::deploy_devnet,
};
use chrono::{DateTime, Utc};
use curl::easy::Easy;
use std::{
    fs::{self, create_dir, File, OpenOptions},
    io::Write,
    path::Path,
    process,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub struct Fuzzer {
    /// Shared fuzzing statistics between threads
    pub stats: Arc<Mutex<Statistics>>,
    /// Number of cores/threads
    pub cores: i32,
    /// Contract JSON path
    pub contract_file: String,
    /// Contract ABI JSON path
    pub contract_abi_file: Option<String>,
    /// Devnet Host
    pub devnet_host: Option<String>,
    // Devnet port
    pub devnet_port: Option<String>,
    /// Contract JSON content
    pub contract_content: String,
    /// Contract function to fuzz
    pub functions: Vec<Function>,
    /// Store local/on-disk logs
    pub logs: Option<String>,
    /// Enable fuzzer logs on stdout
    pub stdout: bool,
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
        println!("\t\t\t\t\t\t\tStdout: {}", config.stdout);
        println!("\t\t\t\t\t\t\tLog file: {}", config.logs);

        let seed = match config.seed {
            Some(val) => val,
            None => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Failed to get actual time")
                .as_millis() as u64,
        };
        println!("\t\t\t\t\t\t\tSeed: {}", seed);

        // Read contract JSON artifact and get its content
        let contents = fs::read_to_string(&config.contract_file)
            .expect("Should have been able to read the file");
        let functions = if config.cairo {
            match parse_json(&contents, &config.function_name) {
                Some(func) => vec![func],
                None => {
                    println!("Could not parse json artifact properly");
                    process::exit(1);
                }
            }
        } else {
            parse_starknet_json(&contents, &config.function_name)
        };

        // Load inputs from the input file if provided
        // TODO need to fix for multiple functions
        let inputs: InputFile = match config.input_file.is_empty() && config.input_folder.is_empty()
        {
            true => InputFile::new_from_function(&functions[0], &config.workspace),
            false => match config.input_folder.is_empty() {
                true => InputFile::load_from_file(&config.input_file, &config.workspace),
                false => InputFile::load_from_folder(&config.input_folder, &config.workspace),
            },
        };
        println!("\t\t\t\t\t\t\tInputs loaded {}", inputs.inputs.len());

        // Load existing inputs in shared database
        if inputs.inputs.len() > 0 {
            let mut stats_db = stats.lock().expect("Failed to lock stats mutex");
            for input in &inputs.inputs {
                if stats_db.input_db.insert(Arc::new(input.clone())) {
                    stats_db.input_list.push(Arc::new(input.clone()));
                    stats_db.input_len += 1;
                }
            }
        }

        // Load crashes from the crash file if provided
        // TODO need to fix for multiple functions
        let crashes: CrashFile =
            match config.crash_file.is_empty() && config.crash_folder.is_empty() {
                true => CrashFile::new_from_function(&functions[0], &config.workspace),
                false => match config.input_folder.is_empty() {
                    true => CrashFile::load_from_file(&config.input_file, &config.workspace),
                    false => CrashFile::load_from_folder(&config.input_folder, &config.workspace),
                },
            };

        // Load existing crashes in shared database
        if crashes.crashes.len() > 0 {
            let mut stats_db = stats.lock().expect("Failed to lock stats mutex");
            for input in &crashes.crashes {
                stats_db.crash_db.insert(Arc::new(input.clone()));
                stats_db.crashes += 1;
            }
        }

        // Setup the mutex for the inputs corpus and crash corpus
        let inputs = Arc::new(Mutex::new(inputs));
        let crashes = Arc::new(Mutex::new(crashes));

        let d = SystemTime::now();
        let datetime = DateTime::<Utc>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d--%H:%M:%S").to_string();
        let _ = create_dir(&config.workspace);
        let _ = create_dir(format!("{}/{}", &config.workspace, &config.function_name));
        let mut file: Option<String> = None;
        if config.logs {
            file = Some(format!(
                "{}/{}/logs_{}.txt",
                &config.workspace, &config.function_name, timestamp_str
            ));
        }
        // Setup the fuzzer
        Fuzzer {
            // Init stats struct
            contract_abi_file: config.abi_path.clone(),
            devnet_host: config.devnet_host.clone(),
            devnet_port: config.devnet_port.clone(),
            stats: stats,
            cores: config.cores,
            logs: file,
            stdout: config.stdout,
            run_time: config.run_time,
            replay: config.replay,
            minimizer: config.minimizer,
            contract_file: config.contract_file.clone(),
            contract_content: contents,
            functions: functions,
            // Init starting time
            start_time: Instant::now(),
            seed: seed,
            input_file: inputs,
            crash_file: crashes,
            workspace: config.workspace.clone(),
            running_workers: 0,
        }
    }

    pub fn starknet_fuzz(&mut self) {
        if let (Some(contract_abi_file), Some(devnet_host), Some(devnet_port)) = (
            &self.contract_abi_file,
            &self.devnet_host,
            &self.devnet_port,
        ) {
            let mut easy = Easy::new();
            easy.url(&format!("http://{}:{}", &devnet_host, &devnet_port))
                .unwrap();
            match easy.perform() {
                Ok(()) => {
                    println!("DEVNET IS UP AT {}:{}", &devnet_host, &devnet_port);
                }
                Err(_) => {
                    println!(
                        "Running another devnet AT {}:{}",
                        &devnet_host, &devnet_port
                    );
                    deploy_devnet(devnet_host.to_string(), devnet_port.to_string());
                }
            }
            let starknet_fuzzer: starknet_helper::starknet::StarknetFuzzer =
                starknet_helper::starknet::StarknetFuzzer::new(
                    &self.contract_file,
                    contract_abi_file,
                    &format!("http://{}:{}", &devnet_host, &devnet_port),
                );
            let stats = self.stats.clone();
            let starknet_fuzzer_clone = starknet_fuzzer.clone();
            let all_functions = self.functions.clone();
            let seed = self.seed + (1 as u64); // create unique seed per worker
            let _ = std::thread::spawn(move || {
                let worker =
                    StarknetWorker::new(stats, 1, starknet_fuzzer_clone, all_functions, seed);
                worker.fuzz();
            });
            self.running_workers += 1;
            self.monitor();
        }
    }

    /// Fuzz
    pub fn fuzz(&mut self) {
        // Running all the threads
        for i in 0..self.cores {
            // create dedicated statistics per thread
            let stats = self.stats.clone();
            let contract_content = self.contract_content.clone();
            let function = self.functions[0].clone();
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
        let stats_db = self.stats.lock().expect("Failed to lock stats mutex");
        // Load inputs
        let mut corpus = stats_db.input_list.clone();
        println!("Total inputs to replay => {}", corpus.len());
        // Load crashes
        let mut crashes = stats_db.crash_db.clone().into_iter().collect();
        corpus.append(&mut crashes);
        drop(stats_db);
        // Split the inputs into chunks
        let chunk_size = match corpus.len() > (self.cores as usize) {
            true => corpus.len() / (self.cores as usize),
            false => 1,
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
            let function = self.functions[0].clone();
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
            let stats = self.stats.lock().expect("Failed to lock stats mutex");
            // Init the struct
            let mut dump_inputs = InputFile {
                workspace: self.workspace.clone(),
                path: format!("{}_min.json", self.functions[0].name),
                name: self.functions[0].name.clone(),
                args: self.functions[0].type_args.clone(),
                inputs: Vec::<Vec<Felt>>::new(),
            };
            // Push every input to the struct
            for input in &stats.input_db {
                dump_inputs.inputs.push(input.clone().to_vec());
            }
            println!("Size after minimization : {}", dump_inputs.inputs.len());
            // Dump the struct
            dump_inputs.dump_json();
        }
    }

    fn logger(&self, content: &String) {
        match self.logs.clone() {
            Some(log) => match fs::metadata(log.clone()) {
                Ok(_) => {
                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(&log)
                        .unwrap();
                    file.write_all(content.as_bytes()).unwrap();
                }
                Err(_e) => {
                    let filename = Path::new(&log);
                    let mut file = File::create(filename).unwrap();
                    write!(file, "{}", content).expect("Failed to write logs in log file");
                }
            },
            None => (),
        }
        match self.stdout {
            true => println!("{}", content),
            false => (),
        }
    }

    /// Function to print stats of the running fuzzer
    fn monitor(&self) {
        // Monitoring loop
        loop {
            // wait 1 second
            std::thread::sleep(Duration::from_millis(1000));

            // Get uptime
            let uptime = (Instant::now() - self.start_time).as_secs_f64();

            // Get access to the global stats
            {
                let stats = self.stats.lock().expect("Failed to lock stats mutex");

                // number of executions
                let fuzz_case = stats.fuzz_cases;
                self.logger(&format!(
                    "{:12.2} uptime | {:9} fuzz cases | {:12.2} fcps | \
                            {:6} coverage | {:6} inputs | {:6} crashes [{:6} unique]\n",
                    uptime,
                    fuzz_case,
                    fuzz_case as f64 / uptime,
                    stats.coverage_db.len(),
                    stats.input_len,
                    stats.crashes,
                    stats.crash_db.len()
                ));
                // Writing inside logging file
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
        assert_eq!(fuzzer.functions[0].name, "test_symbolic_execution");
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
            abi_path: None,
            devnet_host: None,
            devnet_port: None,
            cairo: true,
            starknet: false,
            input_folder: input_folder,
            crash_folder: crash_folder,
            workspace,
            contract_file,
            function_name,
            input_file,
            crash_file,
            cores,
            logs,
            stdout: true,
            seed,
            run_time,
            replay,
            minimizer,
        };
        let fuzzer = Fuzzer::new(&config);
        assert_eq!(fuzzer.cores, 1);
        assert_eq!(fuzzer.functions[0].name, "test_symbolic_execution");
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
            abi_path: None,
            devnet_host: None,
            devnet_port: None,
            cairo: true,
            starknet: false,
            input_folder: input_folder,
            crash_folder: crash_folder,
            workspace,
            contract_file,
            function_name,
            input_file,
            crash_file,
            cores,
            logs,
            stdout: true,
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
        let input_file: String =
            "tests/test_symbolic_execution_2022-12-22--10:18:57.json".to_string();
        let crash_file: String = "".to_string();
        let workspace: String = "fuzzer_workspace".to_string();
        let input_folder: String = "".to_string();
        let crash_folder: String = "".to_string();
        let config = Config {
            abi_path: None,
            devnet_host: None,
            devnet_port: None,
            cairo: true,
            starknet: false,
            input_folder: input_folder,
            crash_folder: crash_folder,
            workspace,
            contract_file,
            function_name,
            input_file,
            crash_file,
            cores,
            logs,
            stdout: true,
            seed,
            run_time,
            replay,
            minimizer,
        };
        // create the fuzzer
        let mut fuzzer = Fuzzer::new(&config);

        fuzzer.replay();

        let stats = fuzzer.stats.lock().expect("Failed to lock stats mutex");
        assert_ne!(stats.coverage_db.len(), 0);
    }
}
