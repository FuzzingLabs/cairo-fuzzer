use crate::fuzzer::coverage::Coverage;
use crate::fuzzer::crash::Crash;
use crate::fuzzer::error::Error;
use crate::fuzzer::stats::Stats;
use crate::mutator::mutator::Mutator;
use crate::mutator::rng::Rng;
use crate::mutator::types::Type;
use crate::runner::runner::Runner;
use crate::runner::starknet_runner;
use bichannel::Channel;
use cairo_vm::Felt252;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[derive(Clone)]
pub enum WorkerEvent {
    NewCrash(Vec<Type>, Error),
    NewUniqueCrash(Crash),
    CoverageUpdateRequest(HashSet<Coverage>),
    CoverageUpdateResponse(HashSet<Coverage>),
}

pub struct Worker {
    channel: Channel<WorkerEvent, WorkerEvent>,
    stats: Arc<RwLock<Stats>>,
    runner: Box<dyn Runner>,
    mutator: Box<dyn Mutator>,
    coverage_set: HashSet<Coverage>,
    unique_crashes_set: HashSet<Crash>,
    statefull: bool,
    rng: Rng,
    diff_fuzz: bool,
    execs_before_cov_update: u64,
}

impl Worker {
    pub fn new(
        channel: Channel<WorkerEvent, WorkerEvent>,
        stats: Arc<RwLock<Stats>>,
        coverage_set: HashSet<Coverage>,
        runner: Box<dyn Runner>,
        mutator: Box<dyn Mutator>,
        seed: u64,
        statefull: bool,
        diff_fuzz: bool,
        execs_before_cov_update: u64,
    ) -> Self {
        Worker {
            channel,
            stats,
            runner,
            mutator,
            coverage_set,
            unique_crashes_set: HashSet::new(),
            statefull,
            rng: Rng {
                seed,
                exp_disabled: false,
            },
            diff_fuzz: diff_fuzz,
            execs_before_cov_update,
        }
    }

    fn pick_and_mutate_inputs(&mut self) -> Vec<Type> {
        let cov = self
            .coverage_set
            .iter()
            .nth(self.rng.rand(0, self.coverage_set.len() - 1))
            .unwrap();
        self.mutator.mutate(&cov.inputs, 2)
    }

    fn init_inputs(inputs: Vec<Type>) -> Vec<Type> {
        let mut res = vec![];
        for param in inputs {
            res.push(match param {
                Type::Felt252(_) => Type::Felt252(Felt252::from(b'\x00')),
                Type::U8(_) => Type::U8(0x00),
                Type::U16(_) => Type::U16(0x00),
                Type::U32(_) => Type::U32(0x00),
                Type::U64(_) => Type::U64(0x00),
                Type::U128(_) => Type::U128(0x00),
                Type::Bool(_) => Type::Bool(true),
                Type::Vector(t, vec) => Type::Vector(t, Self::init_inputs(vec)),
            })
        }
        res
    }

    pub fn run(&mut self) {
        // Utils for execs per sec
        let mut execs_per_sec_timer = Instant::now();
        let mut sec_elapsed = 0;

        // Input initialization
        let mut inputs = Self::init_inputs(self.runner.get_target_parameters());
        let target_function = self.runner.get_function();
        let contract_class = self.runner.get_contract_class();
        loop {
            if !self.statefull {
                self.runner = Box::new(starknet_runner::RunnerStarknet::new(
                    &contract_class,
                    target_function.clone(),
                    self.diff_fuzz,
                ));
            }
            let exec_result = self.runner.execute(inputs.clone()); //self.runner.execute(inputs.clone());

            self.stats.write().unwrap().execs += 1;

            // Calculate execs_per_sec
            if execs_per_sec_timer.elapsed().as_secs() >= 1 {
                execs_per_sec_timer = Instant::now();
                sec_elapsed += 1;
                let tmp = self.stats.read().unwrap().execs;
                self.stats.write().unwrap().secs_since_last_cov += 1;
                self.stats.write().unwrap().execs_per_sec = tmp / sec_elapsed;
            }
            match exec_result {
                Ok(cov) => {
                    if let Some(coverage) = cov {
                        // Execute all activated detectors
                        if !self.coverage_set.contains(&coverage) {
                            self.coverage_set.insert(coverage.clone());
                            self.stats.write().unwrap().secs_since_last_cov = 0;
                        }
                        if coverage.failure {
                            let crash = Crash::new(
                                &self.runner.get_target_module(),
                                &self.runner.get_target_function(),
                                &inputs,
                                &Error::Abort {
                                    message: ("Failure flag").to_string(),
                                },
                            );
                            if !self.unique_crashes_set.contains(&crash) {
                                self.channel
                                    .send(WorkerEvent::NewCrash(
                                        inputs.clone(),
                                        Error::Abort {
                                            message: ("Failure flag").to_string(),
                                        },
                                    ))
                                    .unwrap();
                            }

                            self.stats.write().unwrap().crashes += 1;
                        }
                    }
                }
                Err((coverage, error)) => {
                    // Execute all activated detectors
                    if !self.coverage_set.contains(&coverage) {
                        self.coverage_set.insert(coverage);
                    }
                    self.stats.write().unwrap().secs_since_last_cov = 0;
                    // Might be wring location for this (maybe outside the if)
                    let crash = Crash::new(
                        &self.runner.get_target_module(),
                        &self.runner.get_target_function(),
                        &inputs,
                        &error,
                    );
                    if !self.unique_crashes_set.contains(&crash) {
                        self.channel
                            .send(WorkerEvent::NewCrash(inputs.clone(), error))
                            .unwrap();
                    }

                    self.stats.write().unwrap().crashes += 1;
                }
            }

            // Handle coverage updates every execs_before_cov_update execs (configurable in
            // configfile)
            if self.stats.read().unwrap().execs % self.execs_before_cov_update == 0 {
                self.channel
                    .send(WorkerEvent::CoverageUpdateRequest(
                        self.coverage_set.clone(),
                    ))
                    .unwrap();
            }

            // Check channel from main thread response
            if let Ok(response) = self.channel.try_recv() {
                match response {
                    WorkerEvent::CoverageUpdateResponse(coverage_set) => {
                        self.coverage_set.extend(coverage_set)
                    }
                    WorkerEvent::NewUniqueCrash(crash) => {
                        let _ = self.unique_crashes_set.insert(crash);
                    }
                    _ => unreachable!(),
                }
            }

            // Updates input
            if self.coverage_set.len() > 0 {
                inputs = self.pick_and_mutate_inputs();
            }
        }
    }
}
