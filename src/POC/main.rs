use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use std::process;
use std::process::Command;
use std::process::Output;
use std::sync::Arc;
use std::thread;

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
    let class_hash = regex_class_hash
        .captures(&cmd_output)
        .unwrap()
        .get(1)
        .map_or("", |m| m.as_str());
    return class_hash.to_string();
}

pub fn get_contract_address(out: Output) -> String {
    let cmd_output = &String::from_utf8(out.stdout).unwrap();
    let regex_contract_address = Regex::new(r"Contract address: (.*)").unwrap();
    let contract_address = regex_contract_address
        .captures(&cmd_output)
        .unwrap()
        .get(1)
        .map_or("", |m| m.as_str());
    return contract_address.to_string();
}

pub fn get_tx_hash(out: Output) -> String {
    let cmd_output = &String::from_utf8(out.stdout).unwrap();
    let regex_tx_hash = Regex::new(r"Transaction hash: (.*)").unwrap();
    let tx_hash = regex_tx_hash
        .captures(&cmd_output)
        .unwrap()
        .get(1)
        .map_or("", |m| m.as_str());
    return tx_hash.to_string();
}

pub fn display_output(out: Output) {
    let out_to_str = &String::from_utf8(out.stdout).unwrap();
    println!("STDOUT =>{}", out_to_str);
    let out_to_str = &String::from_utf8(out.stderr).unwrap();
    println!("STDERR =>{}", out_to_str);
}

pub fn deploy_contract(port: &String, rnd_account: &String, contract_path: &String) {
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
    //display_output(declare_contract.clone());
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

pub fn main() {
    let port = "5051";
    let contract_path = &"/home/nabih/cairo-fuzzer/tests/increase_balance.json".to_string();
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
    deploy_contract(&port.to_string(), rnd_account, contract_path);
    println!("Contract deployed successfully");
    while true {}
}
