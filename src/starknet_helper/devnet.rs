use std::{process::Command, sync::Arc, thread};

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
