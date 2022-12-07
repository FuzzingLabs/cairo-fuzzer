use std::collections::hash_map::DefaultHasher;
use std::fs::create_dir;
use std::fs::write;
use std::hash::{Hash, Hasher};

pub fn record_input(fuzz_input: &Vec<u8>, crash: bool) {
    let mut hasher = DefaultHasher::new();
    fuzz_input.hash(&mut hasher);
    if !crash {
        let _ = create_dir("inputs");
        write(
            format!("inputs/{:016x}.input", hasher.finish()),
            format!("{:#?}", fuzz_input),
        )
        .expect("Failed to save input to disk");
    } else {
        let _ = create_dir("crash");
        write(
            format!("crash/{:016x}.input", hasher.finish()),
            format!("{:#?}", fuzz_input),
        )
        .expect("Failed to save input to disk");
    }
}
