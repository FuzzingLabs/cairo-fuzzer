use std::fs;

use crate::cairo_vm::cairo_runner::runner;
use crate::fuzzer::stats::Statistics;
use crate::FuzzingData;

pub fn minimizer(mut stats: Statistics, fuzzing_data: FuzzingData, folder: String) {
    // Local stats database
    let contents = &fuzzing_data.contents;
    let function = &fuzzing_data.function;
    let files = fs::read_dir(folder.to_owned()).unwrap();
    let total = fs::read_dir(folder.to_owned()).unwrap().count();
    let mut index: usize = 0;
    for file in files {
        println!("FILE {} / {} -- ", index, total);
        let input = fs::read_to_string(&file.as_ref().unwrap().path().to_str().unwrap())
            .unwrap()
            .as_bytes()
            .to_vec();
        match runner(&contents, &function.name, &input) {
            Ok(traces) => {
                let mut vec_trace: Vec<(u32, u32)> = vec![];
                for trace in traces.unwrap() {
                    vec_trace.push((
                        trace.0.offset.try_into().unwrap(),
                        trace.1.offset.try_into().unwrap(),
                    ));
                }
                if !stats.coverage_minimizer_db.contains_key(&vec_trace) {
                    stats.input_minimizer_db.insert(input.to_owned());
                    stats
                        .coverage_minimizer_db
                        .insert(vec_trace.clone(), input.to_owned());
                    stats.input_len += 1;
                    print!("KEEPING INPUT");
                } else {
                    stats.removed_files += 1;
                    fs::remove_file(&file.as_ref().unwrap().path()).expect("Failed to delete file");
                    print!("REMOVING INPUT");
                }
            }
            Err(e) => {
                if !stats.crash_minimizer_list.contains(&e.to_string()) {
                    stats.crash_minimizer_list.push(e.to_string());
                    print!("KEEPING INPUT");
                } else {
                    stats.removed_files += 1;
                    fs::remove_file(&file.as_ref().unwrap().path()).expect("Failed to delete file");
                    print!("REMOVING INPUT");
                }
            }
        }
        index += 1;
    }
}
