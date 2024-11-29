use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use cairo_lang_compiler::CompilerConfig;
use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_lang_starknet::compile::compile_path;
use cairo_native::context::NativeContext;
use cairo_native::executor::JitNativeExecutor;
use starknet_types_core::felt::Felt;

use crate::fuzzer::statistics::FuzzerStats;
use crate::fuzzer::utils::{
    find_entry_point_id, get_function_argument_types, print_contract_functions, print_init_message,
};
use crate::mutator::argument_type::ArgumentType;
use crate::mutator::basic_mutator::Mutator;
use crate::runner::runner::{compile_sierra_program, create_executor, run_program};

use log::{error, info, warn};

/// Struct representing the fuzzer
pub struct Fuzzer {
    // Path to the Cairo program
    program_path: PathBuf,
    // Entry point of the Sierra program
    entry_point: Option<String>,
    // Sierra program
    sierra_program: Option<Arc<Program>>,
    // Entry point parameters
    params: Arc<Mutex<Vec<Felt>>>,
    // ID of the entry point
    entry_point_id: Option<FunctionId>,
    // Mutator for the parameters
    mutator: Option<Mutex<Mutator>>,
    // Types of the entry point arguments
    argument_types: Vec<ArgumentType>,
    // Fuzzer statistics
    stats: Arc<Mutex<FuzzerStats>>,
    // Native context
    native_context: NativeContext,
}

impl Fuzzer {
    /// Creates a new Fuzzer.
    pub fn new(program_path: PathBuf, entry_point: Option<String>) -> Self {
        let native_context = NativeContext::new();

        Self {
            program_path,
            entry_point,
            sierra_program: None,
            params: Arc::new(Mutex::new(Vec::new())),
            entry_point_id: None,
            mutator: None,
            argument_types: Vec::new(),
            stats: Arc::new(Mutex::new(FuzzerStats::default())),
            native_context,
        }
    }

    /// Initialize the fuzzer with a given seed
    /// - Initializes the mutator with the given seed
    /// - Compiles Cairo code to Sierra
    /// - Finds the entry point ID
    pub fn init(&mut self, seed: u64) -> Result<(), String> {
        print_init_message(seed);

        self.mutator = Some(Mutex::new(Mutator::new(seed)));

        info!("Compiling Cairo contract to Sierra");
        self.convert_and_store_cairo_to_sierra()?;
        if let Some(ref entry_point) = self.entry_point {
            self.entry_point_id = Some(find_entry_point_id(&self.sierra_program, entry_point));
        }

        Ok(())
    }

    /// Print the contract functions prototypes
    pub fn print_functions_prototypes(&self) {
        println!();
        print_contract_functions(&self.sierra_program);
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

    /// Generates parameters based on the function argument types.
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

    /// Initializes parameters based on the function argument types.
    fn initialize_parameters(&mut self) {
        self.argument_types =
            get_function_argument_types(&self.sierra_program, &self.entry_point_id);
        self.generate_params();
    }

    /// Sets up the execution environment by compiling the Sierra program and creating a JIT executor.
    fn setup_execution_environment(&self) -> Result<JitNativeExecutor, String> {
        let mlir_module = compile_sierra_program(
            &self.native_context,
            self.sierra_program
                .as_ref()
                .ok_or("Sierra program not available")?,
        )?;
        let executor = create_executor(mlir_module);
        Ok(executor)
    }

    /// Executes the program and checks for crashes.
    fn execute_program(&self, executor: &JitNativeExecutor) -> Result<bool, String> {
        let params_guard = self.params.lock().unwrap();
        match run_program(
            executor,
            self.entry_point_id.as_ref().unwrap(),
            &params_guard,
        ) {
            Ok(result) => {
                // Crash detected
                if result.failure_flag 
                    // Ignore this error 
                    && result.error_msg != Some("Failed to deserialize param #1".to_string())
                {
                    // Print the parameters
                    println!("Parameters at crash: {:?}", *params_guard);
                    // Print the result
                    println!("Results : {:?}\n", result);
                    return Ok(true);
                }
            }
            Err(e) => error!("Error during execution: {}", e),
        }
        Ok(false)
    }

    /// Mutates the parameters using the mutator.
    fn mutate_parameters(&self) {
        let mut mutator_guard = self.mutator.as_ref().unwrap().lock().unwrap();
        for param in self.params.lock().unwrap().iter_mut() {
            *param = mutator_guard.mutate(*param);
        }
    }

    /// Prints the statistics every 1000 executions.
    fn print_statistics(&self, current_iter: i32) {
        if current_iter % 1000 == 0 && current_iter != 0 {
            let stats_guard = self.stats.lock().unwrap();
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
        }
    }

    /// Returns a vector of strings with the entry points
    fn get_entry_points(&self) -> Vec<String> {
        let mut entry_points = Vec::new();

        if let Some(program) = &self.sierra_program {
            for function in &program.funcs {
                if let Some(debug_name) = &function.id.debug_name {
                    entry_points.push(debug_name.to_string());
                }
            }
        }

        entry_points
    }

    /// Runs the fuzzer.
    pub fn fuzz(&mut self, iter: i32) -> Result<(), String> {
        self.initialize_parameters();

        // Initialize the start time
        {
            let mut stats_guard = self.stats.lock().unwrap();
            stats_guard.start_time = Instant::now();
        }

        let mut current_iter = 0;
        let max_iter = if iter == -1 { i32::MAX } else { iter };

        info!("Compiling Sierra contract to MLIR module");
        println!();
        let executor = self.setup_execution_environment()?;

        let log_message = format!("Fuzzing function {}", self.entry_point.clone().unwrap());
        info!("{}", log_message);
        // Main fuzz loop
        loop {
            if current_iter >= max_iter {
                warn!("Maximum iterations reached. Exiting fuzzer.");
                break;
            }

            if self.execute_program(&executor)? {
                // Increment the crashes counter
                {
                    let mut stats_guard = self.stats.lock().unwrap();
                    stats_guard.crashes += 1;
                }
                break;
            }

            // Increment the total_executions counter
            {
                let mut stats_guard = self.stats.lock().unwrap();
                stats_guard.total_executions += 1;
            }

            self.mutate_parameters();
            self.print_statistics(current_iter);

            current_iter += 1;
        }

        Ok(())
    }

    /// Fuzzes all functions that finish with "fuzz_*".
    pub fn fuzz_proptesting(&mut self, iter: i32) -> Result<(), String> {
        let entry_points = self.get_entry_points();
        let mut fuzz_functions = Vec::new();

        // Filters out entry points whose names start with fuzz_
        for entry_point in entry_points {
            let parts: Vec<&str> = entry_point.split("::").collect();
            if let Some(last_part) = parts.last() {
                // Ignore __wrapper__ part 
                let modified_last_part = last_part.trim_start_matches("__wrapper__");

                if modified_last_part.starts_with("fuzz_") {
                    fuzz_functions.push(entry_point.to_string());
                }
            }
        }

        // Fuzz all the filtered entrypoints
        for fuzz_function in fuzz_functions {
            info!("Fuzzing function: {}", fuzz_function);
            self.entry_point = Some(fuzz_function.clone());
            self.entry_point_id = Some(find_entry_point_id(&self.sierra_program, &fuzz_function));
            self.initialize_parameters();

            // Run the fuzzer for the current function
            if let Err(e) = self.fuzz(iter) {
                error!("Error fuzzing function {}: {}", fuzz_function, e);
            }
        }

        Ok(())
    }
}
