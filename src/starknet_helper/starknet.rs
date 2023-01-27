use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use std::process;
use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub struct StarknetFuzzer {
    devnet_address: String,
    rnd_account: String,
    contract_path: String,
    abi_path: String,
    contract_address: String,
}

impl StarknetFuzzer {
    pub fn new(contract_path: &String, abi_path: &String, devnet_address: &String) -> Self {
        let rnd_account: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let mut fuzzer = StarknetFuzzer {
            devnet_address: devnet_address.to_string(),
            rnd_account: rnd_account,
            contract_path: contract_path.to_string(),
            abi_path: abi_path.to_string(),
            contract_address: "".to_string(),
        };
        fuzzer.deploy_account();
        println!("Account deployed");
        fuzzer.deploy_contract();
        println!("Contract deployed");
        return fuzzer;
    }

    pub fn crash_check(&self, out: Output) -> bool {
        let out_to_str = &String::from_utf8(out.stderr).unwrap();
        return out_to_str.is_empty();
    }

    /*     pub fn display_tx_status(&self, hash: &String) {
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
            .arg(&self.devnet_address)
            .arg("--gateway_url")
            .arg(&self.devnet_address)
            .output()
            .expect("failed to execute process");
        self.display_output(status);
    } */

    pub fn display_output(&self, out: Output) {
        let out_to_str = &String::from_utf8(out.stdout).unwrap();
        println!("STDOUT =>{}", out_to_str);
        let out_to_str = &String::from_utf8(out.stderr).unwrap();
        println!("STDERR =>{}", out_to_str);
    }

    pub fn get_account_address(&self, out: Output) -> String {
        let cmd_output = &String::from_utf8(out.stdout).unwrap();
        let regex_account_address = Regex::new(r"Account address: (.*)").unwrap();
        let account_address = regex_account_address
            .captures(&cmd_output)
            .unwrap()
            .get(1)
            .map_or("", |m| m.as_str());
        return account_address.to_string();
    }

    pub fn get_class_hash(&self, out: Output) -> String {
        let cmd_output = &String::from_utf8(out.stdout).unwrap();
        let regex_class_hash = Regex::new(r"Contract class hash: (.*)").unwrap();
        if let Some(class_hash) = regex_class_hash.captures(&cmd_output) {
            return class_hash.get(1).map_or("", |m| m.as_str()).to_string();
        }
        let cmd_err = &String::from_utf8(out.stderr).unwrap();
        println!("Error get_class_hash {}", cmd_err);
        process::exit(1);
    }

    pub fn get_contract_address(&self, out: Output) -> String {
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

    pub fn get_tx_hash(&self, out: Output) -> String {
        let cmd_output = &String::from_utf8(out.stdout).unwrap();
        let regex_tx_hash = Regex::new(r"Transaction hash: (.*)").unwrap();
        if let Some(tx_hash) = regex_tx_hash.captures(&cmd_output) {
            return tx_hash.get(1).map_or("", |m| m.as_str()).to_string();
        }
        let cmd_err = &String::from_utf8(out.stderr).unwrap();
        println!("Error get_tx_hash{}", cmd_err);
        process::exit(1);
    }

    pub fn check_tx_status(&self, hash: &String) -> bool {
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
            .arg(&self.devnet_address)
            .arg("--gateway_url")
            .arg(&self.devnet_address)
            .output()
            .expect("failed to execute process");
        let out_to_str = &String::from_utf8(status.stdout).unwrap();
        return out_to_str.contains("ACCEPTED");
    }

    pub fn deploy_contract(&mut self) {
        let declare_contract = Command::new("starknet")
            .env(
                "STARKNET_WALLET",
                "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
            )
            .env("STARKNET_NETWORK", "alpha-goerli")
            .arg("declare")
            .arg("--contract")
            .arg(&self.contract_path)
            .arg("--account")
            .arg(&self.rnd_account)
            .arg("--feeder_gateway_url")
            .arg(&self.devnet_address)
            .arg("--gateway_url")
            .arg(&self.devnet_address)
            .output()
            .expect("failed to execute process");

        let declare_hash = &self.get_tx_hash(declare_contract.clone());
        if self.check_tx_status(declare_hash) {
            let class_hash = self.get_class_hash(declare_contract.clone());
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
                .arg(&self.rnd_account)
                .arg("--feeder_gateway_url")
                .arg(&self.devnet_address)
                .arg("--gateway_url")
                .arg(&self.devnet_address)
                .output()
                .expect("failed to execute process");
            let deploy_hash = &self.get_tx_hash(deploy_contract.clone());
            if !self.check_tx_status(deploy_hash) {
                println!("Error while deploying contract");
                process::exit(1);
            }
            self.contract_address = self.get_contract_address(deploy_contract.clone());
        } else {
            println!("Error while declaring contract");
            process::exit(1);
        }
    }

    pub fn deploy_account(&self) {
        let new_account = Command::new("starknet")
            .env(
                "STARKNET_WALLET",
                "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
            )
            .env("STARKNET_NETWORK", "alpha-goerli")
            .arg("new_account")
            .arg("--account")
            .arg(&self.rnd_account)
            .output()
            .expect("failed to execute process");

        let account_address = self.get_account_address(new_account);
        let curl_addr = format!("{}/mint", &self.devnet_address);
        let curl_content = format!(
            "{{ \"address\": \"{}\", \"amount\": 1000000000000000000, \"lite\": false }}",
            &account_address
        );
        let _feed_account = Command::new("curl")
            .arg(curl_addr)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(curl_content)
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
            .arg(&self.rnd_account)
            .arg("--feeder_gateway_url")
            .arg(&self.devnet_address)
            .arg("--gateway_url")
            .arg(&self.devnet_address)
            .output()
            .expect("failed to execute process");
        let deploy_hash = &self.get_tx_hash(deploy_account.clone());
        if !self.check_tx_status(deploy_hash) {
            println!("Error while deploying account");
            process::exit(1);
        }
    }

    pub fn call_contract(&self, function_name: &String) -> bool {
        let call_contract = Command::new("starknet")
            .env(
                "STARKNET_WALLET",
                "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
            )
            .env("STARKNET_NETWORK", "alpha-goerli")
            .arg("call")
            .arg("--address")
            .arg(&self.contract_address)
            .arg("--abi")
            .arg(&self.abi_path)
            .arg("--function")
            .arg(function_name)
            .arg("--account")
            .arg(&self.rnd_account)
            .arg("--feeder_gateway_url")
            .arg(&self.devnet_address)
            .arg("--gateway_url")
            .arg(&self.devnet_address)
            .output()
            .expect("failed to execute process");
        //self.display_output(call_contract.clone());
        return self.crash_check(call_contract);

        //self.display_output(call_contract.clone());
    }

    pub fn invoke_contract(&self, function_name: &String, inputs: &String) -> bool {
        println!("invoking");
        println!("{:?}", &self);
        let invoke_contract = Command::new("starknet")
            .env(
                "STARKNET_WALLET",
                "starkware.starknet.wallets.open_zeppelin.OpenZeppelinAccount",
            )
            .env("STARKNET_NETWORK", "alpha-goerli")
            .arg("invoke")
            .arg("--address")
            .arg(&self.contract_address)
            .arg("--abi")
            .arg(&self.abi_path)
            .arg("--function")
            .arg(function_name)
            .arg("--inputs")
            .arg(inputs)
            .arg("--account")
            .arg(&self.rnd_account)
            .arg("--feeder_gateway_url")
            .arg(&self.devnet_address)
            .arg("--gateway_url")
            .arg(&self.devnet_address)
            .output()
            .expect("failed to execute process");
        self.display_output(invoke_contract.clone());
        //self.display_output(invoke_contract.clone());
        let invoke_hash = &self.get_tx_hash(invoke_contract.clone());
        if !self.check_tx_status(invoke_hash) {
            return false;
        }
        return self.crash_check(invoke_contract.clone());
    }
}
