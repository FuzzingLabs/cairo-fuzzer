mod custom_rand;
mod fuzzer;
mod mutator;
mod runner;
mod utils;

use clap::Parser;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::fuzzer::fuzzer::Fuzzer;

/// Command-line arguments for the fuzzer
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the Cairo program
    #[arg(short, long)]
    program_path: PathBuf,

    /// Entry point of the Sierra program
    #[arg(short, long)]
    entry_point: Option<String>,

    /// Analyze the program and print function prototypes
    #[arg(short, long, requires = "program_path")]
    analyze: bool,

    /// Number of iterations to use for fuzzing
    #[arg(short, long)]
    iter: Option<i32>,

    /// Enable property-based testing
    #[arg(long)]
    proptesting: bool,

    /// Seed for the random number generator
    #[arg(short, long)]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();

    // Initialize the logger
    colog::init();

    // Determine the seed value
    let seed = args.seed.unwrap_or_else(|| {
        // Use the current time as default seed if the --seed parameter is not specified
        let start = SystemTime::now();
        start
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get the current time")
            .as_secs()
    });

    // Set the default value for iter based on proptesting flag
    let iter = if args.proptesting {
        args.iter.unwrap_or(10000)
    } else {
        args.iter.unwrap_or(-1)
    };

    // Check if --entry-point parameter is required
    if !(args.proptesting || args.analyze) && args.entry_point.is_none() {
        eprintln!("Error: --entry-point is required if --proptesting is not set");
        return;
    }

    // Init the fuzzer
    let mut fuzzer = Fuzzer::new(args.program_path, args.entry_point);

    match fuzzer.init(seed) {
        Ok(()) => {
            // Print the contract functions
            if args.analyze {
                fuzzer.print_functions_prototypes();
            }
            // Run the fuzzer
            else {
                if args.proptesting {
                    match fuzzer.fuzz_proptesting(iter) {
                        Ok(()) => println!("Property-based testing completed successfully."),
                        Err(e) => eprintln!("Error during property-based testing: {}", e),
                    }
                } else {
                    match fuzzer.fuzz(iter) {
                        Ok(()) => println!("Fuzzing completed successfully."),
                        Err(e) => eprintln!("Error during fuzzing: {}", e),
                    }
                }
            }
        }
        Err(e) => eprintln!("Error during initialization: {}", e),
    }
}
