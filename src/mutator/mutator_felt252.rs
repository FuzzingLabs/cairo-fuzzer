use super::types::Type;
use crate::mutator::basicmutator::{self, EmptyDatabase};
use crate::mutator::mutator::Mutator;
use crate::runner::starknet_runner::convert_calldata;

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
    fn fix_inputs_types(&mut self, inputs: &Vec<Type>) -> Vec<Type> {
        let mut idx = 0;
        let mut res: Vec<Type> = vec![];
        inputs.into_iter().for_each(|input_type| {
            match input_type {
                Type::U8(_) => {
                    let bits_to_save = 8;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_bytes_be();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }
                        res.push(Type::U8(u8::from_be_bytes(
                            new_value[new_value.len() - 1..].try_into().unwrap(),
                        )));
                    }
                }
                Type::U16(_) => {
                    let bits_to_save = 16;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_bytes_be();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }

                        res.push(Type::U16(u16::from_be_bytes(
                            new_value[new_value.len() - 2..].try_into().unwrap(),
                        )));
                    }
                }
                Type::U32(_) => {
                    let bits_to_save = 32;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_bytes_be();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }

                        res.push(Type::U32(u32::from_be_bytes(
                            new_value[new_value.len() - 4..].try_into().unwrap(),
                        )));
                    }
                }
                Type::U64(_) => {
                    let bits_to_save = 64;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_bytes_be();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }

                        res.push(Type::U64(u64::from_be_bytes(
                            new_value[new_value.len() - 8..].try_into().unwrap(),
                        )));
                    }
                }
                Type::U128(_) => {
                    let bits_to_save = 128;
                    if bits_to_save != 256 && bits_to_save != 252 {
                        let mut new_value = self.mutator.input[idx].to_bytes_be();
                        for i in 0..new_value.len() - (bits_to_save / 8) {
                            new_value[i] = 0;
                        }

                        res.push(Type::U128(u128::from_be_bytes(
                            new_value[new_value.len() - 16..].try_into().unwrap(),
                        )));
                    }
                }
                Type::Felt252(_) => {
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
