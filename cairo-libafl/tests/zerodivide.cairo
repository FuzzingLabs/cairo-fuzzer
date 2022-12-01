%builtins output

from starkware.cairo.common.serialize import serialize_word

func divide{output_ptr: felt*}() -> (res : felt) {
    let res = 10;
    return (res=res);
}

func main{output_ptr : felt*}(){
    
    let (value) = return_10();

    serialize_word(value);

    return ();
}