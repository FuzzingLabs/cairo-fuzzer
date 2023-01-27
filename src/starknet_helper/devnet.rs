//! This module contains functions to run and deploy a devnet
use std::{process::Command, sync::Arc, thread};

/// Function used to deploy the devnet
pub fn deploy_devnet(address: String, port: String) {
    thread::spawn(move || {
        Arc::new(
            Command::new("starknet-devnet")
                .env("STARKNET_DEVNET_CAIRO_VM", "rust")
                .arg("-p")
                .arg(port.clone())
                .arg("--host")
                .arg(address.clone())
                .output()
                .expect("failed to execute process"),
        );
    });
}
