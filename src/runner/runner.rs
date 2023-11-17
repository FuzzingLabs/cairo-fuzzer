use felt::Felt252;
use starknet_rs::execution::CallInfo;

pub trait Runner {
    fn run(self, data: &Vec<Felt252>) -> Result<(Self, CallInfo), String>
    where
        Self: Sized;
}
