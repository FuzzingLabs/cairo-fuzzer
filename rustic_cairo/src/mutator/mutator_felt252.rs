use crate::mutator::basicmutator::{self, EmptyDatabase};
use crate::mutator::mutator::Mutator;
use crate::runner::starknet_runner::convert_calldata;
use felt::Felt252;

use super::types::Type;

pub struct CairoMutator {
    mutator: basicmutator::Mutator,
}

impl CairoMutator {
    pub fn new(seed: u64, max_input_size: usize) -> Self {
        let mutator = basicmutator::Mutator::new()
            .seed(seed)
            .max_input_size(max_input_size);
        CairoMutator { mutator }
    }
}

impl Mutator for CairoMutator {
    /* fn mutate(&mut self, inputs: &Vec<Type>, nb_mutation: usize) -> Vec<Type> {
        let mut res = vec![];
        //eprintln!("mutating =>");
        self.mutator.input.clear();
        for input in inputs {
            //eprintln!("input: {:?}", input);
            //eprintln!("mutator.input: {:?}", self.mutator.input);
            match input {
                Type::U8(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::U16(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::U32(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::U64(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::U128(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::Bool(b) => self
                    .mutator
                    .input
                    .extend_from_slice(&[if *b { 1 } else { 0 }]),
                Type::Felt252(v) => self.mutator.input.extend_from_slice(&v.to_be_bytes()),
                Type::Vector(_, vec) => {
                    let buffer: Vec<u8> = vec
                        .iter()
                        .map(|v| {
                            if let Type::U8(a) = v {
                                a.to_owned()
                            } else {
                                todo!()
                            }
                        })
                        .collect();
                    self.mutator.input.extend_from_slice(&buffer);
                }
            }

            self.mutator.mutate(nb_mutation, &EmptyDatabase);

            // The size of the input needs to be the right size
            res.push(match input {
                Type::U8(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(1, 0);

                    Type::U8(u8::from_be_bytes(v[0..1].try_into().unwrap()))
                }
                Type::U16(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(1, 0);

                    Type::U8(u8::from_be_bytes(v[0..1].try_into().unwrap()))
                    /*                     let mut v = self.mutator.input.clone();
                    v.resize(2, 0);

                    Type::U16(u16::from_be_bytes(v[0..2].try_into().unwrap())) */
                }
                Type::U32(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(1, 0);

                    Type::U8(u8::from_be_bytes(v[0..1].try_into().unwrap()))
                    /*                     let mut v = self.mutator.input.clone();
                    v.resize(4, 0);

                    Type::U32(u32::from_be_bytes(v[0..4].try_into().unwrap())) */
                }
                Type::U64(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(1, 0);

                    Type::U8(u8::from_be_bytes(v[0..1].try_into().unwrap()))
                    /*                     let mut v = self.mutator.input.clone();
                    v.resize(8, 0);

                    Type::U64(u64::from_be_bytes(v[0..8].try_into().unwrap())) */
                }
                Type::U128(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(1, 0);

                    Type::U8(u8::from_be_bytes(v[0..1].try_into().unwrap()))
                    /*                     let mut v = self.mutator.input.clone();
                    v.resize(16, 0);

                    Type::U128(u128::from_be_bytes(v[0..16].try_into().unwrap())) */
                }
                Type::Felt252(_) => {
                    let mut v = self.mutator.input.clone();
                    v.resize(1, 0);

                    Type::U8(u8::from_be_bytes(v[0..1].try_into().unwrap()))
                    /*                     let mut v = self.mutator.input.clone();
                    v.resize(252, 0);

                    Type::Felt252(Felt252::from_bytes_be(v[0..32].try_into().unwrap())) */
                }
                Type::Bool(_) => Type::Bool(self.mutator.input[0] != 0),
                Type::Vector(_, _) => Type::Vector(
                    Box::new(Type::U8(0)),
                    self.mutator
                        .input
                        .iter()
                        .map(|a| Type::U8(a.to_owned()))
                        .collect(),
                ),
            });
        }
        res
    } */
    /*     fn fix_inputs_types(&mut self, inputs: &Vec<Type>) -> Vec<Type> {
        let mut idx = 0;
        let mut res: Vec<Type> = vec![];
        inputs.into_iter().for_each(|input_type| {
            match input_type {
                Type::U8(_) => {
                    let new_value = self.mutator.input[idx].to_be_bytes();
                    res.push(Type::U8(u8::from_be_bytes(
                        new_value[0..1].try_into().unwrap(),
                    )));
                }
                Type::U16(_) => {
                    let new_value = self.mutator.input[idx].to_be_bytes();
                    res.push(Type::U16(u16::from_be_bytes(
                        new_value[0..2].try_into().unwrap(),
                    )));
                }
                Type::U32(_) => {
                    let new_value = self.mutator.input[idx].to_be_bytes();
                    res.push(Type::U32(u32::from_be_bytes(
                        new_value[0..4].try_into().unwrap(),
                    )));
                }
                Type::U64(_) => {
                    let new_value = self.mutator.input[idx].to_be_bytes();
                    res.push(Type::U64(u64::from_be_bytes(
                        new_value[0..8].try_into().unwrap(),
                    )));
                }
                Type::U128(_) => {
                    let new_value = self.mutator.input[idx].to_be_bytes();
                    res.push(Type::U128(u128::from_be_bytes(
                        new_value[0..16].try_into().unwrap(),
                    )));
                }
                Type::Felt252(_) => {
                    let new_value = self.mutator.input[idx].to_be_bytes();
                    res.push(Type::Felt252(Felt252::from_bytes_be(
                        new_value[0..32].try_into().unwrap(),
                    )));
                }
                _ => {
                    todo!()
                }
            }
            idx += 1;
        });
        res
    } */
    fn fix_inputs_types(&mut self, inputs: &Vec<Type>) -> Vec<Type> {
        let mut idx = 0;
        let mut res: Vec<Type> = vec![];
        inputs.into_iter().for_each(|input_type| {
            match input_type {
                Type::U8(_) => {
                    let bits_to_save = 8;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_be_bytes();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }
                        /*                         eprintln!("new_value u8: {:?}", new_value);
                        eprintln!(
                            "converted new_value: {:?}",
                            u8::from_be_bytes(new_value[new_value.len() - 1..].try_into().unwrap(),)
                        ); */
                        res.push(Type::U8(u8::from_be_bytes(
                            new_value[new_value.len() - 1..].try_into().unwrap(),
                        )));
                    }
                }
                Type::U16(_) => {
                    let bits_to_save = 16;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_be_bytes();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }
                        /*                         eprintln!("new_value u16: {:?}", new_value);
                        eprintln!(
                            "converted new_value: {:?}",
                            u16::from_be_bytes(
                                new_value[new_value.len() - 2..].try_into().unwrap(),
                            )
                        ); */
                        res.push(Type::U16(u16::from_be_bytes(
                            new_value[new_value.len() - 2..].try_into().unwrap(),
                        )));
                    }
                }
                Type::U32(_) => {
                    let bits_to_save = 32;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_be_bytes();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }
                        /*                         eprintln!("new_value u32: {:?}", new_value);
                        eprintln!(
                            "converted new_value: {:?}",
                            u32::from_be_bytes(
                                new_value[new_value.len() - 4..].try_into().unwrap(),
                            )
                        ); */
                        res.push(Type::U32(u32::from_be_bytes(
                            new_value[new_value.len() - 4..].try_into().unwrap(),
                        )));
                    }
                }
                Type::U64(_) => {
                    let bits_to_save = 64;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_be_bytes();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }
                        /*                         eprintln!("new_value u64: {:?}", new_value);
                        eprintln!(
                            "converted new_value: {:?}",
                            u64::from_be_bytes(
                                new_value[new_value.len() - 8..].try_into().unwrap(),
                            )
                        ); */
                        res.push(Type::U64(u64::from_be_bytes(
                            new_value[new_value.len() - 8..].try_into().unwrap(),
                        )));
                    }
                }
                Type::U128(_) => {
                    let bits_to_save = 128;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_be_bytes();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }
                        /*                         eprintln!("new_value u128: {:?}", new_value);
                        eprintln!(
                            "converted new_value: {:?}",
                            u128::from_be_bytes(
                                new_value[new_value.len() - 16..].try_into().unwrap(),
                            )
                        ); */
                        res.push(Type::U128(u128::from_be_bytes(
                            new_value[new_value.len() - 16..].try_into().unwrap(),
                        )));
                    }
                }
                Type::Felt252(_) => {
                    let bits_to_save = 252;

                    res.push(Type::Felt252(self.mutator.input[idx].clone()));
                }
                _ => {
                    todo!()
                }
            }
            idx += 1;
        });
        res
    }
    fn mutate(&mut self, inputs: &Vec<Type>, nb_mutation: usize) -> Vec<Type> {
        self.mutator.input.clear();
        let felt252_converted = convert_calldata(inputs.to_vec());
        self.mutator.input.extend_from_slice(&felt252_converted);
        self.mutator.mutate(nb_mutation, &EmptyDatabase);

        self.fix_inputs_types(inputs)
    }
}
