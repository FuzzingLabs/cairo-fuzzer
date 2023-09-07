use felt::Felt252;
use starknet_rs::execution::CallInfo;

pub trait Runner {
    fn runner(self, data: &Vec<Felt252>) -> Result<(Self, CallInfo), String>
    where
        Self: Sized;
}
