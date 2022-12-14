use serde::Serialize;
use std::fs::create_dir;
use std::fs::write;

use crate::{InputCorpus, CrashCorpus};


/* pub fn record_input(fuzz_input: &Vec<u8>, crash: bool) {
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
 */

 pub fn record_json_input(inputs_corpus: &InputCorpus) {
    let crash_folder = "crashes_corpus";
    let input_folder = "inputs_corpus";
    let _ = create_dir(crash_folder);
    let _ = create_dir(input_folder);
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");

    let mut inputs_ser = serde_json::Serializer::with_formatter(buf.clone(), formatter.clone());
    inputs_corpus.serialize(&mut inputs_ser).unwrap();
    write(
        format!("{}/{}_inputs.json",input_folder, inputs_corpus.name),
        String::from_utf8(inputs_ser.into_inner()).unwrap(),
    )
    .expect("Failed to save input to disk");
}

pub fn record_json_crash(crashes_corpus: &CrashCorpus) {
    let crash_folder = "crashes_corpus";
    let input_folder = "inputs_corpus";
    let _ = create_dir(crash_folder);
    let _ = create_dir(input_folder);
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    // TODO - Change name of crashes files by adding the date and the time

    let mut crashes_ser = serde_json::Serializer::with_formatter(buf.clone(), formatter.clone());
    crashes_corpus.serialize(&mut crashes_ser).unwrap();
    write(
        format!("{}/{}_crashes.json",crash_folder, crashes_corpus.name),
        String::from_utf8(crashes_ser.into_inner()).unwrap(),
    )
    .expect("Failed to save input to disk");
}
