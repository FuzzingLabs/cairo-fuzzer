use clap::{self, Parser};

#[derive(Debug, Parser)]
pub struct Opt {
    #[arg(
        long,
        help = "Set the number of threads to run",
        name = "CORES",
        default_value = "1"
    )]
    pub cores: i32,

    #[arg(
        long,
        help = "Set the path of the JSON artifact to load",
        name = "CONTRACT",
        default_value = ""
    )]
    pub contract: String,

    #[arg(
        long,
        help = "Set the path of the JSON CASM artifact to load",
        name = "CASM",
        default_value = ""
    )]
    pub casm: String,

    #[arg(
        long,
        help = "Set the function to fuzz",
        name = "target_function",
        default_value = ""
    )]
    pub target_function: String,
    #[arg(
        long,
        help = "Keep the state of the fuzzer between runs",
        name = "STATEFULL",
        default_value = "false"
    )]
    pub statefull: bool,
    #[arg(
        long,
        help = "Workspace of the fuzzer",
        name = "WORKSPACE",
        default_value = "fuzzer_workspace"
    )]
    pub workspace: String,

    #[arg(
        long,
        help = "Path to the inputs folder to load",
        name = "corpus_dir",
        default_value = "./corpus_dir"
    )]
    pub corpus_dir: String,

    #[arg(
        long,
        help = "Path to the crashes folder to load",
        name = "crashes_dir",
        default_value = "./crash_dir"
    )]
    pub crashes_dir: String,

    #[arg(
        long,
        help = "Path to the inputs file to load",
        name = "INPUTFILE",
        default_value = ""
    )]
    pub inputfile: String,

    #[arg(
        long,
        help = "Path to the crashes file to load",
        name = "CRASHFILE",
        default_value = ""
    )]
    pub crashfile: String,

    #[arg(
        long,
        help = "Path to the dictionnary file to load",
        name = "DICT",
        default_value = ""
    )]
    pub dict: String,

    #[arg(
        long,
        help = "Enable fuzzer logs in file",
        name = "LOGS",
        default_value = "false"
    )]
    pub logs: bool,

    #[arg(
        long,
        help = "Set a custom seed (only applicable for 1 core run)",
        name = "SEED",
        default_value = "0"
    )]
    pub seed: Option<u64>,

    #[arg(
        long,
        help = "Number of seconds this fuzzing session will last",
        name = "RUN_TIME"
    )]
    pub run_time: Option<u64>,

    #[arg(long, help = "Load config file", name = "CONFIG")]
    pub config: Option<String>,

    #[arg(
        long,
        help = "Replay the corpus folder",
        name = "REPLAY",
        default_value = "false"
    )]
    pub replay: bool,
    #[arg(
        long,
        help = "Minimize Corpora",
        name = "MINIMIZER",
        default_value = "false"
    )]
    pub minimizer: bool,
    #[arg(
        long,
        help = "Property Testing",
        name = "PROPTESTING",
        default_value = "false"
    )]
    pub proptesting: bool,

    #[arg(
        long,
        help = "Dump functions prototypes",
        name = "ANALYZE",
        default_value = "false"
    )]
    pub analyze: bool,
    #[arg(long, help = "Iteration Number", name = "ITER", default_value = "-1")]
    pub iter: i64,
}
