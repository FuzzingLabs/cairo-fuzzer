use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use cairo_lang_compiler::CompilerConfig;
use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_lang_starknet::compile::compile_path;
use colored::*;
use starknet_types_core::felt::Felt;

use crate::fuzzer::statistics::FuzzerStats;
use crate::mutator::argument_type::map_argument_type;
use crate::mutator::argument_type::ArgumentType;
use crate::mutator::basic_mutator::Mutator;
use crate::runner::runner::CairoNativeRunner;
use crate::utils::get_function_by_id;

#[warn(dead_code)]
pub struct Fuzzer {
    program_path: PathBuf,
    entry_point: Option<String>,
    runner: Arc<Mutex<CairoNativeRunner>>,
    sierra_program: Option<Arc<Program>>,
    params: Arc<Mutex<Vec<Felt>>>,
    entry_point_id: Option<FunctionId>,
    mutator: Option<Mutex<Mutator>>,
    argument_types: Vec<ArgumentType>,
    stats: Arc<Mutex<FuzzerStats>>,
}

impl Fuzzer {
    /// Creates a new `Fuzzer`.
    pub fn new(program_path: PathBuf, entry_point: Option<String>) -> Self {
        Self {
            program_path,
            entry_point,
            runner: Arc::new(Mutex::new(CairoNativeRunner::new())),
            sierra_program: None,
            params: Arc::new(Mutex::new(Vec::new())),
            entry_point_id: None,
            mutator: None,
            argument_types: Vec::new(),
            stats: Arc::new(Mutex::new(FuzzerStats::default())),
        }
    }

    /// Init the fuzzer with a given seed
    /// - Initialize the mutator with the given seed
    /// - Compile Cairo code to Sierra
    /// - Find the entry id
    /// - Init the runner
    pub fn init(&mut self, seed: u64) -> Result<(), String> {
        println!(
            "
=============================================================================================================================================================
╔═╗ ┌─┐ ┬ ┬─┐ ┌───┐   ╔═╗ ┬ ┬ ┌─┐ ┌─┐ ┌─┐ ┬─┐      | Seed -- {}
║   ├─┤ │ ├┬┘ │2.0│───╠╣  │ │ ┌─┘ ┌─┘ ├┤  ├┬┘      | 
╚═╝ ┴ ┴ ┴ ┴└─ └───┘   ╚   └─┘ └─┘ └─┘ └─┘ ┴└─      |
=============================================================================================================================================================\n",
            seed,
        );

        println!("[+] Initializing mutator with seed: {}", seed);
        self.mutator = Some(Mutex::new(Mutator::new(seed)));

        println!("[+] Compiling Cairo contract to Sierra");
        self.convert_and_store_cairo_to_sierra()?;
        if let Some(ref entry_point) = self.entry_point {
            self.entry_point_id = Some(self.find_entry_point_id(entry_point));
        }

        println!("[+] Initializing the runner");
        self.runner
            .lock()
            .unwrap()
            .init(&self.entry_point_id, &self.sierra_program)?;

        Ok(())
    }

    /// Print the contract functions prototypes
    pub fn print_functions_prototypes(&self) {
        println!("Contract functions :\n");

        for function in &self.sierra_program.clone().unwrap().funcs {
            let function_name = function
                .id
                .debug_name
                .as_ref()
                .map_or_else(|| "unknown".to_string(), |name| name.to_string());

            let signature = &function.signature;

            // Collect parameter types
            let param_types: Vec<String> = signature
                .param_types
                .iter()
                .map(|param| {
                    param
                        .debug_name
                        .as_ref()
                        .map_or_else(|| "unknown".to_string(), |name| name.to_string())
                })
                .collect();

            // Collect return types
            let ret_types: Vec<String> = signature
                .ret_types
                .iter()
                .map(|ret_type| {
                    ret_type
                        .debug_name
                        .as_ref()
                        .map_or_else(|| "unknown".to_string(), |name| name.to_string())
                })
                .collect();

            // Format the prototype
            let prototype = format!(
                "{} ({}) -> ({})",
                function_name.bold().white(),
                param_types.join(", ").green(),
                ret_types.join(", ").cyan()
            );

            // Print the contract functions
            println!("- {}", prototype);
        }
    }

    /// Compile the Cairo program to Sierra
    fn convert_and_store_cairo_to_sierra(&mut self) -> Result<(), String> {
        let contract = compile_path(
            &self.program_path,
            None,
            CompilerConfig {
                replace_ids: true,
                ..Default::default()
            },
        )
        .map_err(|e| format!("Failed to compile Cairo program: {}", e))?;

        let sierra_program = contract
            .extract_sierra_program()
            .map_err(|e| format!("Failed to extract Sierra program: {}", e))?;
        self.sierra_program = Some(Arc::new(sierra_program));
        Ok(())
    }

    /// Find the entry point id
    fn find_entry_point_id(&self, entry_point: &str) -> FunctionId {
        let sierra_program = self
            .sierra_program
            .as_ref()
            .expect("Sierra program not available");
        cairo_native::utils::find_function_id(sierra_program, entry_point)
            .expect(&format!("Entry point '{}' not found", entry_point))
            .clone()
    }

    /// Returns a vector of the function parameter types
    ///
    /// For example, given a function with the prototype:
    /// ```
    /// myfunction(a: felt252, b: felt252) -> felt252
    /// ```
    /// This function will return:
    /// ```
    /// [Felt, Felt]
    /// ```
    pub fn get_function_arguments_types(&self) -> Vec<ArgumentType> {
        let func = match (&self.sierra_program, &self.entry_point_id) {
            (Some(program), Some(entry_point_id)) => get_function_by_id(program, entry_point_id),
            _ => None,
        };

        if let Some(func) = func {
            let argument_types: Vec<ArgumentType> = func
                .signature
                .param_types
                .iter()
                .filter_map(|param_type| {
                    if let Some(debug_name) = &param_type.debug_name {
                        // Map param_type to an `ArgumentType`
                        // For now we only handle felt252
                        return map_argument_type(debug_name);
                    }
                    None
                })
                .collect();
            argument_types
        } else {
            Vec::new()
        }
    }

    /// Generate params based on the function argument types
    pub fn generate_params(&mut self) {
        let mut params = self.params.lock().unwrap();
        *params = self
            .argument_types
            .iter()
            .map(|arg_type| match arg_type {
                ArgumentType::Felt => Felt::from(0),
                // TODO: Add support for other types
            })
            .collect();
    }

    /// Run the fuzzer with multithreading
    /// We use the specified number of threads
    pub fn fuzz(&mut self, num_threads: usize) -> Result<(), String> {
        self.argument_types = self.get_function_arguments_types();
        self.generate_params();

        // Initialize the start time
        {
            let mut stats_guard = self.stats.lock().unwrap();
            stats_guard.start_time = Instant::now();
        }

        // Collect thread handles here
        let mut handles = Vec::new();

        for _ in 0..num_threads {
            // Clone the necessary data for each thread
            let runner = Arc::clone(&self.runner);
            let argument_types = self.argument_types.clone();
            let mutator = Arc::new(Mutex::new(
                self.mutator.as_ref().unwrap().lock().unwrap().clone(),
            ));

            // Clone stats
            let stats = Arc::clone(&self.stats);

            // Generate initial params for each thread
            let params = Arc::new(Mutex::new(
                argument_types
                    .iter()
                    .map(|arg_type| match arg_type {
                        ArgumentType::Felt => Felt::from(0),
                        // TODO: Add support for other types
                    })
                    .collect::<Vec<Felt>>(),
            ));

            // Spawn the thread
            let handle = thread::spawn(move || {
                loop {
                    let params_guard = params.lock().unwrap();
                    match runner.lock().unwrap().run_program(&*params_guard) {
                        Ok(result) => {
                            if result.failure_flag {
                                println!("Results : {:?}", result);
                                println!("Crash detected! Exiting fuzzer.");

                                // Increment the crashes counter
                                {
                                    let mut stats_guard = stats.lock().unwrap();
                                    stats_guard.crashes += 1;
                                }

                                break;
                            }
                        }
                        Err(e) => eprintln!("Error during execution: {}", e),
                    }

                    // Release the lock before mutating params
                    drop(params_guard);

                    // Increment the total_executions counter
                    {
                        let mut stats_guard = stats.lock().unwrap();
                        stats_guard.total_executions += 1;
                    }

                    // Mutate params using the cloned `mutator`
                    let mut mutator_guard = mutator.lock().unwrap();
                    for param in params.lock().unwrap().iter_mut() {
                        *param = mutator_guard.mutate(*param);
                    }
                }
            });

            // Push handle to a local Vec, not self.threads
            handles.push(handle);
        }

        // Spawn a thread to print stats every second
        let stats = Arc::clone(&self.stats);
        let print_handle = thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            let stats_guard = stats.lock().unwrap();
            let uptime = stats_guard.start_time.elapsed();
            let uptime_secs = uptime.as_secs_f64();

            // Calculate execs per second
            let execs_per_second = if uptime_secs > 0.0 {
                stats_guard.total_executions as f64 / uptime_secs
            } else {
                0.0
            };

            println!(
                "| {:<30} | {:<25} | {:<25} | {:<20} |",
                format!("Total Executions = {}", stats_guard.total_executions),
                format!("Uptime = {:.1}s", uptime_secs),
                format!("Crashes = {}", stats_guard.crashes),
                format!("Exec Speed = {:.2} execs/s", execs_per_second)
            );
        });

        // Wait for threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        // Stop the print thread
        print_handle.join().unwrap();

        Ok(())
    }
}
