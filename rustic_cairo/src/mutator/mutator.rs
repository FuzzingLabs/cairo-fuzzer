use super::types::Type;

pub trait Mutator {

    fn mutate(&mut self, inputs: &Vec<Type>, nb_mutation: usize) -> Vec<Type>;

}
