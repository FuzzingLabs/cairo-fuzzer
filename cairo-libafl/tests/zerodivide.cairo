%builtins output

from starkware.cairo.common.serialize import serialize_word

func divide_imp{output_ptr : felt*}(a,b) -> (res : felt) {
    let test = a + b;
    serialize_word(test);
    let res = 10/a;
    serialize_word(res);
    return (res=res);
}

func divide(a) -> (res : felt) {
    let res = 10/2;
    let res = 10/3;
    let res = 10/5;
    let res = 10/a;
    return (res=res);
}

func main{output_ptr : felt*}(){
    
    let (value) = divide(5);

    serialize_word(value);

    return ();
}