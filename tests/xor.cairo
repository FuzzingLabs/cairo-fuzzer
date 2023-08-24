%lang starknet
%builtins range_check bitwise

from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.bitwise import bitwise_xor
@external
func test_password{bitwise_ptr: BitwiseBuiltin*}(password: felt) -> (res: felt) {
    let (result) = bitwise_xor(12345, password);
    if (result == 19423) {
        return (res=1);
    }
    return (res=0);
}
