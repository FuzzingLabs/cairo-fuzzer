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
        bal:u8
    }
 #[external(v0)]
    fn Fuzz_symbolic_execution(
ref self: ContractState,
    f: felt252,
    u: felt252,
    z: u16,
    z2: u32,
    i: u64,
    n: u128,
    g: u128,
    l: u128,
    a: felt252,
    b: felt252,
    s: u8,
    ) {
        if (f == 'f') {
            if (u == 'u') {
                if (z == 'z') {
                    if (z2 == 'z') {
                        if (i == 'i') {
                            if (n == 'n') {
                                if (g == 'g') {
                                    if (l == 'l') {
                                        if (a == 'a') {
                                            if (b == 'b') {
                                                if (s == 's') {
                                                    assert(1==0 , '!(f & t)');
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return ();
    }
}