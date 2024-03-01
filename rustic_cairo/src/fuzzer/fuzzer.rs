use super::crash::Crash;
use super::fuzzer_utils::load_corpus;
use super::fuzzer_utils::load_crashes;
use super::fuzzer_utils::write_corpusfile;
use super::fuzzer_utils::write_crashfile;
use crate::cli::config::Config;
use crate::fuzzer::coverage::Coverage;
use crate::fuzzer::stats::Stats;
use crate::json_helper::json_parser::get_function_from_json;
use crate::json_helper::json_parser::Function;
use crate::mutator;
use crate::mutator::types::Parameters;
use crate::mutator::types::Type;
use crate::runner::runner::Runner;
use crate::runner::starknet_runner;
use crate::ui::ui::{Ui, UiEvent, UiEventData};
use crate::worker::worker::{Worker, WorkerEvent};
use bichannel::Channel;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs;
use std::process;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;
use time::Duration;

pub struct Fuzzer {
    // Fuzzer configuration
    config: Config,
    // Thread specific stats
    threads_stats: Vec<Arc<RwLock<Stats>>>,
    // Channel to communicate with each threads
    channels: Vec<Channel<WorkerEvent, WorkerEvent>>,
    // Global stats mostly for ui
    global_stats: Stats,
    // Global coverage
    coverage_set: HashSet<Coverage>,
    // Unique crashes set
    unique_crashes_set: HashSet<Crash>,
    // The user interface
    ui: Option<Ui>,
    // The contract Class
    contract_class: CasmContractClass,
    // The contract content
    contract_content: String,
    // The function to target in the contract
    target_function: Function,
    // Parameters of the target function
    target_parameters: Vec<Type>,
    // Max coverage
    max_coverage: usize,
}

impl Fuzzer {
    pub fn new(config: Config) -> Self {
        let nb_threads = config.cores as u8;
        let ui = Some(Ui::new(nb_threads, config.seed.unwrap()));
        let contents = fs::read_to_string(&config.contract_file)
            .expect("Should have been able to read the file");
        let casm_content = fs::read_to_string(&config.casm_file).expect("Could not read casm file");
        let contract_class =
            serde_json::from_str(&casm_content).expect("could not get contractclass");
        let function = match get_function_from_json(&contents, &config.target_function) {
            Some(func) => func,
            None => {
                eprintln!("Error: Could not parse json file");
                process::exit(1)
            }
        };
        let coverage_set = load_corpus(&config.corpus_dir).unwrap_or_default();
        let unique_crashes_set = load_crashes(&config.crashes_dir).unwrap_or_default();
        Fuzzer {
            config,
            threads_stats: vec![],
            channels: vec![],
            global_stats: Stats::new(),
            coverage_set,
            unique_crashes_set,
            ui,
            contract_class: contract_class,
            contract_content: contents,
            target_function: function,
            target_parameters: vec![],
            max_coverage: 0,
        }
    }

    fn start_threads(&mut self) {
        for i in 0..self.config.cores {
            // Creates the communication channel for the fuzzer and worker sides
            let (fuzzer, worker) = bichannel::channel::<WorkerEvent, WorkerEvent>();
            self.channels.push(fuzzer);
            let stats = Arc::new(RwLock::new(Stats::new()));
            self.threads_stats.push(stats.clone());
            // Change here the runner you want to create
            let runner = Box::new(starknet_runner::RunnerStarknet::new(
                &self.contract_class,
                self.target_function.clone(),
                self.config.diff_fuzz,
            ));
            self.target_parameters = runner.get_target_parameters();
            self.max_coverage = runner.get_max_coverage();
            let statefull = self.config.statefull;
            let diff_fuzz = self.config.diff_fuzz;
            // Increment seed so that each worker doesn't do the same thing
            let seed = self.config.seed.expect("could not get seed") + (i as u64);
            let execs_before_cov_update = 10000; //xxx todo //self.config.execs_before_cov_update;
            let mutator = Box::new(mutator::mutator_felt252::CairoMutator::new(
                seed,
                self.target_function.inputs.len(),
            ));
            let coverage_set = self.coverage_set.clone();
            let _ = std::thread::Builder::new()
                .name(format!("Worker {}", i).to_string())
                .spawn(move || {
                    // Creates generic worker and starts it
                    let mut w = Worker::new(
                        worker,
                        stats,
                        coverage_set,
                        runner,
                        mutator,
                        seed,
                        statefull,
                        diff_fuzz,
                        execs_before_cov_update,
                    );
                    w.run();
                });
        }
    }

    fn get_global_execs(&self) -> u64 {
        let mut sum: u64 = 0;
        for i in 0..self.config.cores {
            sum += self.threads_stats[i as usize].read().unwrap().execs;
        }
        sum
    }

    fn get_global_crashes(&self) -> u64 {
        let mut sum: u64 = 0;
        for i in 0..self.config.cores {
            sum += self.threads_stats[i as usize].read().unwrap().crashes;
        }
        sum
    }

    fn broadcast(&self, event: &WorkerEvent) {
        for chan in &self.channels {
            chan.send(event.to_owned()).unwrap();
        }
    }

    pub fn run(&mut self) {
        // Init workers
        self.start_threads();

        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();

        let mut events = VecDeque::new();

        if let Some(ui) = &mut self.ui {
            ui.set_target_infos(
                "Starknet",
                &self.target_function.name,
                &self.target_parameters,
                self.max_coverage,
            );
        }

        let mut new_crash: Option<Crash> = None;

        loop {
            // Sum execs
            self.global_stats.execs = self.get_global_execs();
            self.global_stats.crashes = self.get_global_crashes();

            // Calculate execs_per_sec
            if execs_per_sec_timer.elapsed().as_secs() >= 1 {
                execs_per_sec_timer = Instant::now();
                self.global_stats.execs_per_sec = self.global_stats.execs;
                self.global_stats.time_running += 1;
                self.global_stats.secs_since_last_cov += 1;
                self.global_stats.execs_per_sec =
                    self.global_stats.execs_per_sec / self.global_stats.time_running;
            }

            // Checks channels for new data
            for chan in &self.channels {
                if let Ok(event) = chan.try_recv() {
                    // Creates duration used for the ui
                    let duration =
                        Duration::seconds(self.global_stats.time_running.try_into().unwrap());
                    match event {
                        WorkerEvent::CoverageUpdateRequest(coverage_set) => {
                            // Gets diffrences between the two coverage sets
                            let binding = self.coverage_set.clone();
                            let differences_with_main_thread: HashSet<_> =
                                self.coverage_set.difference(&coverage_set).collect();
                            let differences_with_worker: HashSet<_> =
                                coverage_set.difference(&binding).collect();
                            let mut tmp = HashSet::new();
                            for diff in &differences_with_main_thread.clone() {
                                tmp.insert(diff.to_owned().clone());
                            }
                            // Updates sets
                            if differences_with_main_thread.len() > 0 {
                                chan.send(WorkerEvent::CoverageUpdateResponse(tmp)).unwrap();
                            }
                            // Adds all the coverage to the main coverage_set
                            for diff in &differences_with_worker {
                                if !self.coverage_set.contains(diff) {
                                    write_corpusfile(&self.config.corpus_dir, &diff);
                                    self.coverage_set.insert(diff.to_owned().clone());
                                    self.global_stats.secs_since_last_cov = 0;
                                    self.global_stats.coverage_size += 1;
                                    events.push_front(UiEvent::NewCoverage(UiEventData {
                                        time: duration,
                                        message: format!("{}", Parameters(diff.inputs.clone())),
                                        error: None,
                                    }));
                                }
                            }
                        }
                        WorkerEvent::NewCrash(inputs, error) => {
                            let crash = Crash::new(
                                &self.contract_content,
                                &self.target_function.name,
                                &inputs,
                                &error,
                            );
                            let mut message = format!(
                                "already exists, skipping - {}",
                                Parameters(inputs.clone())
                            );
                            if !self.unique_crashes_set.contains(&crash) {
                                write_crashfile(&self.config.crashes_dir, crash.clone());
                                self.global_stats.unique_crashes += 1;
                                self.unique_crashes_set.insert(crash.clone());
                                message = format!("NEW - {}", Parameters(inputs));
                                new_crash = Some(crash);
                            }
                            events.push_front(UiEvent::NewCrash(UiEventData {
                                time: duration,
                                message,
                                error: Some(error),
                            }));
                        }

                        _ => unimplemented!(),
                    }
                }
            }

            // Broadcasting unique crash to all threads
            if let Some(crash) = &new_crash {
                self.broadcast(&WorkerEvent::NewUniqueCrash(crash.clone()));
                new_crash = None;
            }

            // Run ui
            if self
                .ui
                .as_mut()
                .unwrap()
                .render(&self.global_stats, &events, &self.threads_stats)
            {
                self.ui.as_mut().unwrap().restore_terminal();
                eprintln!("Quitting...");
                break;
            }
        }
    }
}
/* } */
