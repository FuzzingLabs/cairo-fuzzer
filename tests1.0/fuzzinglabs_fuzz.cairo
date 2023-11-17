use starknet::{
    Store, SyscallResult, StorageBaseAddress, storage_read_syscall, storage_write_syscall,
    storage_address_from_base_and_offset
};
use integer::{
    U128IntoFelt252, Felt252IntoU256, Felt252TryIntoU64, U256TryIntoFelt252, u256_from_felt252
};

#[starknet::contract]
mod test_contract {
    #[storage]
    struct Storage {
        bal:u128,
    }
 #[external(v0)]
    fn storage_test(ref self: ContractState) {
        let value = self.bal.read();
        assert(value < 100000, 'value sup to 100k');
    }
}
