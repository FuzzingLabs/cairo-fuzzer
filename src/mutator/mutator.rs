use super::types::Type;

pub trait Mutator {
    fn mutate(&mut self, inputs: &Vec<Type>, nb_mutation: usize) -> Vec<Type>;
    fn fix_inputs_types(&mut self, inputs: &Vec<Type>) -> Vec<Type>;
}
