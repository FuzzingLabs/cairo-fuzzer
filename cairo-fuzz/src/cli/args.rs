use clap::{self, Parser};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Opt {
    #[arg(
        long,
        help = "number of threads to run",
        name = "CORES",
        default_value = "1"
    )]
    pub cores: i32,

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

    #[arg(long, help = "Set the artefact of the contract", name = "ITERATION")]
    pub iteration: Option<u64>,
}
