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

      #[external(v0)]
    fn fuzz_i128(ref self: ContractState, value: i128) -> i128 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_i64(ref self: ContractState, value: i64) -> i64 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_i32(ref self: ContractState, value: i32) -> i32 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_i16(ref self: ContractState, value: i16) -> i16 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_i8(ref self: ContractState, value: i8) -> i8 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_u128(ref self: ContractState, value: u128) -> u128 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_u64(ref self: ContractState, value: u64) -> u64 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_u32(ref self: ContractState, value: u32) -> u32 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_u16(ref self: ContractState, value: u16) -> u16 {
        assert(value != 2, 'fail');
        value
    }

    #[external(v0)]
    fn fuzz_u8(ref self: ContractState, value: u8) -> u8 {
        assert(value != 2, 'fail');
        value
    }
}