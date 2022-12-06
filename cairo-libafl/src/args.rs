use clap::{self, Parser};
use core::time::Duration;
use libafl::bolts::core_affinity::Cores;
use libafl::Error;
use std::path::PathBuf;

fn timeout_from_millis_str(time: &str) -> Result<Duration, Error> {
    Ok(Duration::from_millis(time.parse()?))
}

#[derive(Debug, Parser)]
pub struct Opt {
    #[arg(
        long,
        value_parser = Cores::from_cmdline,
        help = "Spawn a client in each of the provided cores. Broker runs in the 0th core. 'all' to select all available cores. 'none' to run a client without binding to any core. eg: '1,2-4,6' selects the cores 1,2,3,4,6.",
        name = "CORES"
    )]
    pub cores: Cores,

    #[arg(long, help = "Set an initial corpus directory", name = "INPUT")]
    pub input: Vec<PathBuf>,

    #[arg(
        long,
        help = "Set the output directory, default is ./out",
        name = "OUTPUT",
        default_value = "./out"
    )]
    pub output: PathBuf,

    #[arg(
        long,
        help = "Set the output directory, default is ./out",
        name = "FUNCTION"
    )]
    pub function: String,

    #[arg(long, help = "Set the artefact of the contract", name = "CONTRACT")]
    pub contract: PathBuf,

    #[arg(
        long,
        help = "Set the artefact of the contract",
        name = "ITERATION",
        default_value = "-1"
    )]
    pub iteration: i64,

    #[arg(
        value_parser = timeout_from_millis_str,
        long,
        help = "Set the exeucution timeout in milliseconds, default is 10000",
        name = "TIMEOUT",
        default_value = "10000"
    )]
    pub timeout: Duration,
}
