use felt::Felt252;
use std::fs;

#[derive(Debug, Clone, Default)]
pub struct Dict {
    pub inputs: Vec<Felt252>,
}
impl Dict {
    pub fn read_dict(path: &String) -> Dict {
        println!("\t\t\t\t\t\t\tReading and parsing dict: {}", path);
        let contents = fs::read_to_string(path).expect("Could not read dictionnary");
        let lines = contents.split('\n');

        let mut data: Vec<Felt252> = Vec::new();

        for line in lines {
            let mut parts = line.trim().split('=');
            if let Some(_) = parts.next() {
                if let Some(value) = parts.next() {
                    let val: Result<u128, _> = value.to_owned().parse();
                    data.push(Felt252::from(
                        val.expect("could not get u128 from value in dict"),
                    ));
                }
            }
        }
        return Dict { inputs: data };
    }
}
