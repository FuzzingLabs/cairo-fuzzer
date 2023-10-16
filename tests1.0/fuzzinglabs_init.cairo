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
        bal: u8
    }

    #[external(v0)]
    fn init(ref self: ContractState, value: u8) {
        self.bal.write(value);
    }
}
