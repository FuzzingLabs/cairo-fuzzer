use cairo_fuzzer::{cli::config::Config, fuzzer::fuzzer::Fuzzer};

pub fn setup_config() -> Config {
    let devnet_host = "localhost".to_string();
    let devnet_port = "5050".to_string();
    let abi_path = "/home/nabih/cairo-fuzzer/tests/increase_balance_abi.json".to_string();
    let cores: i32 = 1;
    let logs: bool = false;
    let seed: Option<u64> = Some(1000);
    let run_time: Option<u64> = None;
    let replay: bool = false;
    let minimizer: bool = false;
    let contract_file: String = "/home/nabih/cairo-fuzzer/tests/increase_balance.json".to_string();
    let function_name: String = "increase_balance".to_string();
    let input_file: String = "".to_string();
    let crash_file: String = "".to_string();
    let workspace: String = "fuzzer_workspace".to_string();
    let input_folder: String = "".to_string();
    let crash_folder: String = "".to_string();
    let config = Config {
        starknet:true,
        cairo:false,
        devnet_host:Some(devnet_host),
        devnet_port:Some(devnet_port),
        abi_path:Some(abi_path),
        input_folder: input_folder,
        crash_folder: crash_folder,
        workspace,
        contract_file,
        function_name,
        input_file,
        crash_file,
        cores,
        logs,
        stdout: true,
        seed,
        run_time,
        replay,
        minimizer,
    };
    return config;
}

pub fn main() {
    let config = setup_config();
    let mut fuzzer = Fuzzer::new(&config);
    if config.starknet {
        fuzzer.starknet_fuzz();
    }

}