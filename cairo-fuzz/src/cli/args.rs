use clap::{self, Parser};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Opt {
    #[arg(
        long,
        help = "Set the number of threads to run",
        name = "CORES",
        default_value = "1"
    )]
    pub cores: i32,

    #[arg(long, help = "Set the function to fuzz", name = "FUNCTION")]
    pub function: String,

    #[arg(
        long,
        help = "Set the path of the json artifact to load",
        name = "CONTRACT"
    )]
    pub contract: PathBuf,

    #[arg(long, help = "Set a custom seed", name = "SEED")]
    pub seed: Option<u64>,

    #[arg(
        long,
        help = "Enable fuzzer logs in file",
        name = "logs",
        default_value = "false"
    )]
    pub logs: bool,
}
