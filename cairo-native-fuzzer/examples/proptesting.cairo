#[starknet::contract]
mod Echo {
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState, initial_balance: felt252) {
        //panic_with_felt252('panic');
        self.balance.write(initial_balance);
    }

    #[external(v0)]
    fn fuzz_test(ref self: ContractState, value: felt252) -> felt252 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_test2(ref self: ContractState, value: felt252) -> felt252 {
        assert(value != 3, 'fail');
        value
    }
}