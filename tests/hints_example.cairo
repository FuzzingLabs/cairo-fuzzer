%builtins range_check

from starkware.cairo.common.math import assert_nn

func assert_nn_manual_implementation{range_check_ptr}(a) {
    %{
        from starkware.cairo.common.math_utils import assert_integer
        assert_integer(ids.a)
        assert 0 <= ids.a % PRIME < range_check_builtin.bound, f'a = {ids.a} is out of range.'
    %}
    a = [range_check_ptr];
    let range_check_ptr = range_check_ptr + 1;
    return ();
}

func fib(first_element, second_element, n) -> (res: felt) {
    jmp fib_body if n != 0;
    tempvar result = second_element;
    return (second_element,);

    fib_body:
    tempvar y = first_element + second_element;
    return fib(second_element, y, n - 1);
}

func evaluate_fib() {
    // Call fib(1, 1, 10).
    let result: felt = fib(1, 1, 10);

    // Make sure the 10th Fibonacci number is 144.
    assert result = 144;
    return ();
}


func main{range_check_ptr: felt}() {
    let x = 64;
    tempvar y = 64 * 64;
    assert_nn(1);
    assert_nn(x);
    assert_nn(y);

    assert_nn_manual_implementation(1);
    assert_nn_manual_implementation(x);
    assert_nn_manual_implementation(y);

    return ();
}
