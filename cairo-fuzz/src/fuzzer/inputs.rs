use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::fs::create_dir;
use std::fs::write;
use std::hash::{Hash, Hasher};

use crate::FunctionCorpus;

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

pub fn record_json_input(function_corpus: &FunctionCorpus) {
    let _ = create_dir("input_json");
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    function_corpus.serialize(&mut ser).unwrap();
    write(
        format!("input_json/{}.json", function_corpus.name),
        String::from_utf8(ser.into_inner()).unwrap(),
    )
    .expect("Failed to save input to disk");
}
