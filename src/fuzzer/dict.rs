use std::fs;

use felt::Felt252;

use crate::mutator::mutator::InputDatabase;

pub fn read_dict(path: &String) -> Dict {
    println!("\t\t\t\t\t\t\tReading and parsing dict: {}", path);
    let contents = fs::read_to_string(path).expect("Could not read dictionnary");
    let lines = contents.split('\n');

    let mut data: Vec<Felt252> = Vec::new();

    for line in lines {
        let mut parts = line.trim().split('=');
        if let Some(_) = parts.next() {
            if let Some(value) = parts.next() {
                let val : Result<u128, _> = value.to_owned().parse();
                data.push(Felt252::from(val.expect("could not get u128 from value in dict")));
            }
        }
    }
    return Dict { inputs: data };
}

#[derive(Debug, Clone, Default)]
pub struct Dict {
    pub inputs: Vec<Felt252>,
}
impl InputDatabase for Dict {
    fn num_inputs(&self) -> usize {
        return self.inputs.len();
    }
// Dict take Felt252
// Send it to the mutator
// Via  CLI the user will be able to choose between the different mutators
// We will initiate the mutatore choosen by the user
// The mutator will take the dictionnary and will use the values
// the mutator will return a Felt252
    fn input(&self, idx: usize) -> Option<Felt252> {
        let value: Felt252 = self.inputs[idx].clone();
        return Some(value.clone());
    }
}