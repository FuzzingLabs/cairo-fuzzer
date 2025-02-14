
#[starknet::contract]
mod Echo {
    use integer::u8_try_as_non_zero;

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn echo_felt(ref self: ContractState, value: felt252) -> felt252 {
        assert(value != 2, 'fail');
        value
    }
}
