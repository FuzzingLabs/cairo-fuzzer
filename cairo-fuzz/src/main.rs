

use std::fs::File;
use std::io::Write;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono;

use basic_mutator::Mutator;
use basic_mutator::EmptyDatabase;

#[allow(unused)]
use std::cell::Cell;
// use rusty_v8 as v8;

const MAX_THREADS: u32 = 1;

use cairo_vm::cairo_runner::runner;

pub fn target(buf: &[u8]) -> Result<Vec<(u32,u32)>, usize>{
    let mut res: Vec<(u32, u32)> = vec![];

    res.push((0,0));

    if buf.len() == 11 {
        if buf[0] as char == 'f' {
            //dprintln!("f");
            res.push((0,1));

            if buf[1] as char == 'u' {
                //dprintln!("u");
                res.push((0,2));

                if buf[2] as char == 'z' {
                    //dprintln!("z");
                    res.push((0,3));

                    if buf[3] as char == 'z' {
                        //dprintln!("z");
                        res.push((0,4));

                        if buf[4] as char == 'i' {
                            //dprintln!("i");
                            res.push((0,5));

                            if buf[5] as char == 'n' {
                                //dprintln!("n");
                                res.push((0,6));

                                if buf[6] as char == 'g' {
                                    //dprintln!("g");
                                    res.push((0,7));

                                    panic!("gg {:?}", buf);
                                }


                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(res);
}

/// A basic random number generator based on xorshift64 with 64-bits of state
struct Rng {
    /// The RNG's seed and state
    seed: u64,

    /// If set, `rand_exp` behaves the same as `rand`
    exp_disabled: bool,
}

impl Rng {
    /// Generate a random number
    #[inline]
    fn next(&mut self) -> u64 {
        let val = self.seed;
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 43;
        val
    }

    /// Generates a random number with uniform distribution in the range of
    /// [min, max]
    #[inline]
    fn rand(&mut self, min: usize, max: usize) -> usize {
        // Make sure the range is sane
        assert!(max >= min, "Bad range specified for rand()");

        // If there is no range, just return `min`
        if min == max {
            return min;
        }
        
        // If the range is unbounded, just return a random number
        if min == 0 && max == core::usize::MAX {
            return self.next() as usize;
        }

        // Pick a random number in the range
        min + (self.next() as usize % (max - min + 1))
    }
    
    /// Generates a random number with exponential distribution in the range of
    /// [min, max] with a worst case deviation from uniform of 0.5x. Meaning
    /// this will always return uniform at least half the time.
    #[inline]
    fn rand_exp(&mut self, min: usize, max: usize) -> usize {
        // If exponential random is disabled, fall back to uniform
        if self.exp_disabled {
            return self.rand(min, max);
        }

        if self.rand(0, 1) == 0 {
            // Half the time, provide uniform
            self.rand(min, max)
        } else {
            // Pick an exponentially difficult random number
            let x = self.rand(min, max);
            self.rand(min, x)
        }
    }
}


struct Fuzzer {

    cov: Vec<Vec<(u32, u32)>>,
    inputs: Vec<Vec<u8>>,
    /// The random number generator used for mutations
    rng: Rng,
}

impl Fuzzer {

    pub fn new() -> Self {
        Fuzzer {
            cov: Vec::new(),
            inputs: Vec::new(),
            rng: Rng {
                seed:         0x12640367f4b7ea35,
                exp_disabled: false,
            },
        }
    }

    pub fn size_inputs(&self) -> usize {
        return self.inputs.len()
    }

    fn is_inside_coverage(&self, item: &Vec<(u32, u32)>) -> bool {
        if self.cov.contains(&item) { 
            //println!("yes");
            return true;
        } else {
            //println!("no");
            return false;
        }
    }

    fn new_input(&mut self, item: &Vec<u8>){
        self.inputs.push(item.to_vec());
    }

    fn new_cov(&mut self, item: &Vec<(u32, u32)>){
        self.cov.push(item.clone());
    }

    fn get_rnd_input(&mut self) -> Vec<u8> {
        let input_idx: usize = self.rng.rand(0, self.size_inputs()-1);
        return self.inputs[input_idx].clone()
    }
}

use std::process::exit;
use std::{fs, path::PathBuf};
mod cairo_vm;
mod utils;

use utils::parse_json::parse_json;

fn main() {

    let contract = "../cairo-libafl/tests/fuzzinglabs.json";
    let function_name = "test_symbolic_execution";
    // --contract tests/fuzzinglabs.json --function "test_symbolic_execution"
    let contents =
        fs::read_to_string(&contract.to_string()).expect("Should have been able to read the file");
    let function = match parse_json(&contents, &function_name.to_string()) {
        Some(func) => func,
        None => {
            println!("Could not find the function {}", function_name);
            return;
        }
    };


    let now = Instant::now();
    let mut counter:Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    let crash = Arc::new(Mutex::new(0));
    //let mut threads = Vec::new();

    // Create a mutator for 11-byte ASCII printable inputs
    let mut mutator = Mutator::new().seed(1337)
        .max_input_size(11).printable(true);

    // feedback list
    let mut cov_map = Fuzzer::new();
    
    cov_map.new_input(&b"AAAAAAAAAAA".to_vec());

    let mut count: usize = 0; 
    'next_case: loop {

        // clear previous data
        mutator.input.clear();
        // pick from feedback corpora
        mutator.input.extend_from_slice(&cov_map.get_rnd_input());
        // Corrupt it with 4 mutation passes
        mutator.mutate(4, &EmptyDatabase);
        //assert!(mutator.input.len() == 11);
        if mutator.input.len() != 11 {
            continue 'next_case;
        }
        
        //let res = target(&mutator.input).unwrap();

        
        match runner(&contents, function.name.clone(), &mutator.input) {
            Ok(traces) => {
                //println!("traces = {:?}", traces);
                let mut res: Vec<(u32, u32)> = vec![];
                for trace in traces.unwrap() {
                    //signals_set(trace.0.offset * 1000 + trace.1.offset);
                    res.push((trace.0.offset.try_into().unwrap(), trace.1.offset.try_into().unwrap()));


                    //dprintln!("Setting signals! {} {}",trace.0.offset, trace.1.offset);
                }

                // is it new?
                // if not inside current coverage
                if cov_map.is_inside_coverage(&res) == false {
                    // new input
                    println!("new_cov = {:?}", res);
                    cov_map.new_cov(&res);
                    println!("new_input = {:?}", mutator.input);
                    cov_map.new_input(&mutator.input);
                    println!("current cov {:?}", &cov_map.inputs);
                    println!("new cov {:?}", &mutator.input);
                }
            }
            Err(e) => {
                panic!("{:?} {:?}", &mutator.input, e);
            },
        }


        //exit(1);
        //is_inside_coverage(, &res[-1])
        /*
        // detect crash - TODO
        if client.messages.len() >= 2 {
            //println!("{}", client.messages[0]);
            //println!("{}", client.messages[1]);
            //println!("{}", client.messages[2]);
            if client.messages[0] != client.messages[1] && client.messages[1] != client.messages[2] {
                println!("{:?}", source);
                let timestamp = chrono::offset::Local::now();
                println!("CRASH DETECTED at {:?}", timestamp);
                let mut output = File::create(format!("CRASH_{:?}.txt",  timestamp)).unwrap();
                write!(output, "{}", source).unwrap();
                let mut crash = crash.lock().unwrap();
                *crash += 1;
                //std::process::exit(0x1337);
            }
        }
        */
        //println!("result {:?}", client.messages);

        // only update every 100k exec to prevent lock
        if count % 1000 == 1 {
            let mut counter = counter.lock().unwrap();
            println!("{:?}", &mutator.input);
            //*counter += 1000;

            //std::thread::sleep(Duration::from_millis(10000));
            //let counter = counter.lock().unwrap();
            let crash = crash.lock().unwrap();
            //let crash_counter = crash_counter.lock().unwrap();
            println!(
                "total exec {} -- {} fcps -- {} crash",
                count,
                count as f64 / (Instant::now() - now).as_secs_f64(),
                crash
            );
        }
        count += 1;
    }


    println!("Hello, world!");
}


fn worker(counter: Arc<Mutex<u64>>, crash: Arc<Mutex<u64>>, i: u32) {
    println!("Start thread id {}", i);

}