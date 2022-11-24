%builtins output

from starkware.cairo.common.serialize import serialize_word

func get_value{output_ptr: felt*}(integer: felt) {
    tempvar x = ;
    tempvar z = integer; 
    tempvar y = x + z;
    serialize_word(y);
    return();
}

func main{output_ptr: felt*}() {
    vulnerable_function(1);
    return ();
}