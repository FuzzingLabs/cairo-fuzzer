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
        help = "Set the function to fuzz",
        name = "FUNCTION",
        default_value = ""
    )]
    pub function: String,

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
        name = "INPUTFOLDER",
        default_value = ""
    )]
    pub inputfolder: String,

    #[arg(
        long,
        help = "Path to the crashes folder to load",
        name = "CRASHFOLDER",
        default_value = ""
    )]
    pub crashfolder: String,

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
        help = "Enable fuzzer logs in file",
        name = "LOGS",
        default_value = "false"
    )]
    pub logs: bool,

    #[arg(
        long,
        help = "Enable fuzzer logs in STDOUT",
        name = "STDOUT",
        default_value = "false"
    )]
    pub stdout: bool,

    #[arg(
        long,
        help = "Set a custom seed (only applicable for 1 core run)",
        name = "SEED"
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

    #[arg(long, help = "Load ABI file", name = "ABI_PATH")]
    pub abi_path: Option<String>,

    #[arg(long, help = "Devnet IP", name = "DEVNET_HOST")]
    pub devnet_host: Option<String>,

    #[arg(long, help = "Devnet PORT", name = "DEVNET_PORT")]
    pub devnet_port: Option<String>,

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
        help = "Fuzz Starknet",
        name = "STARKNET",
        default_value = "false"
    )]
    pub starknet: bool,

    #[arg(long, help = "Fuzz Cairo", name = "CAIRO", default_value = "false")]
    pub cairo: bool,
}
