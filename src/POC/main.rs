use rand::seq::SliceRandom;
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::process;
use std::process::Command;
use std::process::Output;
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    /// parse entrypoint number in the json
    pub entrypoint: String,
    pub num_args: u64,
    pub type_args: Vec<String>,
    pub hints: bool,
    pub decorators: Vec<String>,
    pub _starknet: bool,
}
pub struct Contract {
    address: String,
}

pub fn display_tx_status(hash: &String, port: &String) {
    let address = format!("http://127.0.0.1:{}", port).to_string();
    let status = Command::new("starknet")
        .env(
            "STARKNET_WALLET",
            "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
        )
        .env("STARKNET_NETWORK", "alpha-goerli")
        .arg("tx_status")
        .arg("--hash")
        .arg(hash)
        .arg("--feeder_gateway_url")
        .arg(&address)
        .arg("--gateway_url")
        .arg(&address)
        .output()
        .expect("failed to execute process");
    display_output(status);
}

pub fn check_tx_status(hash: &String, port: &String) -> bool {
    let address = format!("http://127.0.0.1:{}", port).to_string();
    let status = Command::new("starknet")
        .env(
            "STARKNET_WALLET",
            "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
        )
        .env("STARKNET_NETWORK", "alpha-goerli")
        .arg("tx_status")
        .arg("--hash")
        .arg(hash)
        .arg("--feeder_gateway_url")
        .arg(&address)
        .arg("--gateway_url")
        .arg(&address)
        .output()
        .expect("failed to execute process");
    let out_to_str = &String::from_utf8(status.stdout).unwrap();
    return out_to_str.contains("ACCEPTED");
}

pub fn get_account_address(out: Output) -> String {
    let cmd_output = &String::from_utf8(out.stdout).unwrap();
    let regex_account_address = Regex::new(r"Account address: (.*)").unwrap();
    let account_address = regex_account_address
        .captures(&cmd_output)
        .unwrap()
        .get(1)
        .map_or("", |m| m.as_str());
    return account_address.to_string();
}

pub fn get_class_hash(out: Output) -> String {
    let cmd_output = &String::from_utf8(out.stdout).unwrap();
    let regex_class_hash = Regex::new(r"Contract class hash: (.*)").unwrap();
    if let Some(class_hash) = regex_class_hash.captures(&cmd_output) {
        return class_hash.get(1).map_or("", |m| m.as_str()).to_string();
    }
    let cmd_err = &String::from_utf8(out.stderr).unwrap();
    println!("Error get_class_hash {}", cmd_err);
    process::exit(1);
}

pub fn get_contract_address(out: Output) -> String {
    let cmd_output = &String::from_utf8(out.stdout).unwrap();
    let regex_contract_address = Regex::new(r"Contract address: (.*)").unwrap();
    if let Some(contract_address) = regex_contract_address.captures(&cmd_output) {
        return contract_address
            .get(1)
            .map_or("", |m| m.as_str())
            .to_string();
    }
    let cmd_err = &String::from_utf8(out.stderr).unwrap();
    println!("Error get_contract_address {}", cmd_err);
    process::exit(1);
}

pub fn get_tx_hash(out: Output) -> String {
    let cmd_output = &String::from_utf8(out.stdout).unwrap();
    let regex_tx_hash = Regex::new(r"Transaction hash: (.*)").unwrap();
    if let Some(tx_hash) = regex_tx_hash.captures(&cmd_output) {
        return tx_hash.get(1).map_or("", |m| m.as_str()).to_string();
    }
    let cmd_err = &String::from_utf8(out.stderr).unwrap();
    println!("Error get_tx_hash{}", cmd_err);
    process::exit(1);
}

pub fn display_output(out: Output) {
    let out_to_str = &String::from_utf8(out.stdout).unwrap();
    println!("STDOUT =>{}", out_to_str);
    let out_to_str = &String::from_utf8(out.stderr).unwrap();
    println!("STDERR =>{}", out_to_str);
}

pub fn deploy_contract(port: &String, rnd_account: &String, contract_path: &String) -> Contract {
    let address = format!("http://127.0.0.1:{}", port).to_string();
    let declare_contract = Command::new("starknet")
        .env(
            "STARKNET_WALLET",
            "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
        )
        .env("STARKNET_NETWORK", "alpha-goerli")
        .arg("declare")
        .arg("--contract")
        .arg(contract_path)
        .arg("--account")
        .arg(&rnd_account)
        .arg("--feeder_gateway_url")
        .arg(&address)
        .arg("--gateway_url")
        .arg(&address)
        .output()
        .expect("failed to execute process");

    let declare_hash = &get_tx_hash(declare_contract.clone());
    if check_tx_status(declare_hash, port) {
        let class_hash = get_class_hash(declare_contract.clone());
        let deploy_contract = Command::new("starknet")
            .env(
                "STARKNET_WALLET",
                "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
            )
            .env("STARKNET_NETWORK", "alpha-goerli")
            .arg("deploy")
            .arg("--class_hash")
            .arg(class_hash)
            .arg("--account")
            .arg(&rnd_account)
            .arg("--feeder_gateway_url")
            .arg(&address)
            .arg("--gateway_url")
            .arg(&address)
            .output()
            .expect("failed to execute process");
        let deploy_hash = &get_tx_hash(deploy_contract.clone());
        if !check_tx_status(deploy_hash, port) {
            println!("Error while deploying contract");
            process::exit(1);
        }
        let contract_address = get_contract_address(deploy_contract.clone());
        return Contract {
            address: contract_address,
        };
        //display_tx_status(deploy_hash, port);
    } else {
        println!("Error while declaring contract");
        process::exit(1);
    }
}

pub fn deploy_account(port: &String, rnd_account: &String) {
    let address = format!("http://127.0.0.1:{}", port).to_string();
    let new_account = Command::new("starknet")
        .env(
            "STARKNET_WALLET",
            "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
        )
        .env("STARKNET_NETWORK", "alpha-goerli")
        .arg("new_account")
        .arg("--account")
        .arg(&rnd_account)
        .output()
        .expect("failed to execute process");

    let account_address = get_account_address(new_account);
    let _feed_account = Command::new("/home/nabih/cairo-fuzzer/src/POC/mint.sh")
        .arg(account_address)
        .arg(port)
        .output()
        .expect("Failed to fund account");

    let deploy_account = Command::new("starknet")
        .env(
            "STARKNET_WALLET",
            "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
        )
        .env("STARKNET_NETWORK", "alpha-goerli")
        .arg("deploy_account")
        .arg("--account")
        .arg(&rnd_account)
        .arg("--feeder_gateway_url")
        .arg(&address)
        .arg("--gateway_url")
        .arg(&address)
        .output()
        .expect("failed to execute process");
    let deploy_hash = &get_tx_hash(deploy_account.clone());
    if !check_tx_status(deploy_hash, port) {
        println!("Error while deploying account");
        process::exit(1);
    }
}

pub fn call_contract(
    port: &String,
    rnd_account: &String,
    abi_path: &String,
    contract_data: &Contract,
    function_name: &String,
) {
    let address = format!("http://127.0.0.1:{}", port).to_string();
    let call_contract = Command::new("starknet")
        .env(
            "STARKNET_WALLET",
            "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
        )
        .env("STARKNET_NETWORK", "alpha-goerli")
        .arg("call")
        .arg("--address")
        .arg(&contract_data.address)
        .arg("--abi")
        .arg(abi_path)
        .arg("--function")
        .arg(function_name)
        .arg("--account")
        .arg(&rnd_account)
        .arg("--feeder_gateway_url")
        .arg(&address)
        .arg("--gateway_url")
        .arg(&address)
        .output()
        .expect("failed to execute process");
    display_output(call_contract.clone());
}

pub fn invoke_contract(
    port: &String,
    rnd_account: &String,
    abi_path: &String,
    contract_data: &Contract,
    function_name: &String,
    inputs: &String,
) {
    let address = format!("http://127.0.0.1:{}", port).to_string();
    let invoke_contract = Command::new("starknet")
        .env(
            "STARKNET_WALLET",
            "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
        )
        .env("STARKNET_NETWORK", "alpha-goerli")
        .arg("invoke")
        .arg("--address")
        .arg(&contract_data.address)
        .arg("--abi")
        .arg(abi_path)
        .arg("--function")
        .arg(function_name)
        .arg("--inputs")
        .arg(inputs)
        .arg("--account")
        .arg(&rnd_account)
        .arg("--feeder_gateway_url")
        .arg(&address)
        .arg("--gateway_url")
        .arg(&address)
        .output()
        .expect("failed to execute process");
    let invoke_hash = &get_tx_hash(invoke_contract);
    if !check_tx_status(invoke_hash, port) {
        println!("Error while invoking contract");
        process::exit(1);
    }
}

pub fn generate_tx_sequence(
    all_func: &Vec<Function>,
    port: &String,
    rnd_account: &String,
    abi_path: &String,
    contract_data: &Contract,
) {
    let mut rng = rand::thread_rng();
    let n1: u8 = rng.gen();
    let mut tx_sequence: Vec<Function> = Vec::new();
    for _i in 0..n1 {
        tx_sequence.push(all_func.choose(&mut rng).unwrap().clone());
    }
    for func in tx_sequence {
        if func.decorators.contains(&"view".to_string()) {
            call_contract(port, rnd_account, abi_path, contract_data, &func.name);
        } else {
            let mut inputs: String = "".to_string();
            for _i in 0..func.num_args {
                let value: u8 = rng.gen();
                inputs += &format!("{} ", value).to_string();
            }
            invoke_contract(
                port,
                rnd_account,
                abi_path,
                contract_data,
                &func.name,
                &inputs,
            )
        }
    }
}

pub fn main() {
    let port = "5051";
    let contract_path = &"/home/nabih/cairo-fuzzer/tests/increase_balance.json".to_string();
    let abi_path = &"/home/nabih/cairo-fuzzer/tests/increase_balance_abi.json".to_string();

    let contents = fs::read_to_string(contract_path).expect("Failed to read string from the file");

    let all_func = parse_json(&contents);
    println!("{:?}", all_func);
    let rnd_account: &String = &rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    thread::spawn(move || {
        Arc::new(
            Command::new("starknet-devnet")
                .arg("-p")
                .arg(port.clone())
                .output()
                .expect("failed to execute process")
                .stdout,
        );
    });
    deploy_account(&port.to_string(), rnd_account);
    println!("Account deployed successfully");
    let contract_data = deploy_contract(&port.to_string(), rnd_account, contract_path);
    println!("Contract deployed successfully");
    println!("First call");
    call_contract(
        &port.to_string(),
        rnd_account,
        abi_path,
        &contract_data,
        &"get_balance".to_string(),
    );
    invoke_contract(
        &port.to_string(),
        rnd_account,
        abi_path,
        &contract_data,
        &"increase_balance".to_string(),
        &"1".to_string(),
    );
    call_contract(
        &port.to_string(),
        rnd_account,
        abi_path,
        &contract_data,
        &"get_balance".to_string(),
    );
    println!("Contract invoked");
    while true {
        generate_tx_sequence(
            &all_func,
            &port.to_string(),
            rnd_account,
            abi_path,
            &contract_data,
        )
    }
}
