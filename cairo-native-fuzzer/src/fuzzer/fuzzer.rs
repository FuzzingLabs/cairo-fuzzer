use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::runner::runner::compile_sierra_program;
use crate::runner::runner::create_executor;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra::program::Program;
use cairo_lang_starknet::compile::compile_path;
use cairo_native::context::NativeContext;
use colored::*;
use starknet_types_core::felt::Felt;

use crate::fuzzer::statistics::FuzzerStats;
use crate::mutator::argument_type::{map_argument_type, ArgumentType};
use crate::mutator::basic_mutator::Mutator;
use crate::runner::runner::run_program;
use crate::utils::get_function_by_id;

#[warn(dead_code)]
pub struct Fuzzer {
    program_path: PathBuf,
    entry_point: Option<String>,
    sierra_program: Option<Arc<Program>>,
    params: Arc<Mutex<Vec<Felt>>>,
    entry_point_id: Option<FunctionId>,
    mutator: Option<Mutex<Mutator>>,
    argument_types: Vec<ArgumentType>,
    stats: Arc<Mutex<FuzzerStats>>,
    native_context: NativeContext,
}

/// Print the initial message with the seed
fn print_init_message(seed: u64) {
    println!(
        "
=============================================================================================================================================================
╔═╗ ┌─┐ ┬ ┬─┐ ┌───┐   ╔═╗ ┬ ┬ ┌─┐ ┌─┐ ┌─┐ ┬─┐      | Seed -- {}
║   ├─┤ │ ├┬┘ │2.0│───╠╣  │ │ ┌─┘ ┌─┘ ├┤  ├┬┘      |
╚═╝ ┴ ┴ ┴ ┴└─ └───┘   ╚   └─┘ └─┘ └─┘ └─┘ ┴└─      |
=============================================================================================================================================================\n",
        seed,
    );
}

/// Print the contract functions prototypes
fn print_contract_functions(sierra_program: &Option<Arc<Program>>) {
    println!("Contract functions :\n");

    if let Some(program) = sierra_program {
        for function in &program.funcs {
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
}

/// Find the entry point id
fn find_entry_point_id(sierra_program: &Option<Arc<Program>>, entry_point: &str) -> FunctionId {
    let sierra_program = sierra_program
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
fn get_function_argument_types(
    sierra_program: &Option<Arc<Program>>,
    entry_point_id: &Option<FunctionId>,
) -> Vec<ArgumentType> {
    let func = match (sierra_program, entry_point_id) {
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

impl Fuzzer {
    /// Creates a new `Fuzzer`.
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

    /// Init the fuzzer with a given seed
    /// - Initialize the mutator with the given seed
    /// - Compile Cairo code to Sierra
    /// - Find the entry id
    pub fn init(&mut self, seed: u64) -> Result<(), String> {
        print_init_message(seed);

        println!("[+] Initializing mutator with seed: {}", seed);
        self.mutator = Some(Mutex::new(Mutator::new(seed)));

        println!("[+] Compiling Cairo contract to Sierra");
        self.convert_and_store_cairo_to_sierra()?;
        if let Some(ref entry_point) = self.entry_point {
            self.entry_point_id = Some(find_entry_point_id(&self.sierra_program, entry_point));
        }

        Ok(())
    }

    /// Print the contract functions prototypes
    pub fn print_functions_prototypes(&self) {
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

    /// Run the fuzzer
    pub fn fuzz(&mut self, iter: i32) -> Result<(), String> {
        self.argument_types =
            get_function_argument_types(&self.sierra_program, &self.entry_point_id);
        self.generate_params();

        // Initialize the start time
        {
            let mut stats_guard = self.stats.lock().unwrap();
            stats_guard.start_time = Instant::now();
        }

        let mut current_iter = 0;
        let max_iter = if iter == -1 { i32::MAX } else { iter };

        // Compile Sierra program into a MLIR module
        let mlir_module = compile_sierra_program(
            &self.native_context,
            self.sierra_program
                .as_ref()
                .ok_or("Sierra program not available")?,
        )?;

        // Create a JIT executor
        let executor = create_executor(mlir_module);

        // Infinite loop of execution
        loop {
            if current_iter >= max_iter {
                println!("Maximum iterations reached. Exiting fuzzer.");
                break;
            }

            let params_guard = self.params.lock().unwrap();

            // Execute the program
            match run_program(
                &executor,
                self.entry_point_id.as_ref().unwrap(),
                &params_guard,
            ) {
                Ok(result) => {
                    if result.failure_flag {
                        println!("Results : {:?}", result);
                        println!("Crash detected! Exiting fuzzer.");

                        // Increment the crashes counter
                        {
                            let mut stats_guard = self.stats.lock().unwrap();
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
                let mut stats_guard = self.stats.lock().unwrap();
                stats_guard.total_executions += 1;
            }

            // Mutate params using the mutator
            let mut mutator_guard = self.mutator.as_ref().unwrap().lock().unwrap();
            for param in self.params.lock().unwrap().iter_mut() {
                *param = mutator_guard.mutate(*param);
            }

            // Print stats every 1000 executions
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

            current_iter += 1;
        }

        Ok(())
    }
}
