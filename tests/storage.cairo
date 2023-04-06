%lang starknet
from starkware.cairo.common.cairo_builtins import HashBuiltin
from starkware.cairo.common.math_cmp import is_le

@storage_var
func _counter() -> (res: felt) {
}

@external
func write_and_read{syscall_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr}(arg: felt) -> (
    res: felt
) {
    _counter.write(arg);
    assert arg = 100;
    _counter.write(arg);
    assert is_le(arg,300) = 1;
    return _counter.read();
}
