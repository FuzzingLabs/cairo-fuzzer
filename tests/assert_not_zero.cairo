func assert_not_zero(value: felt) {
    %{
        print("test")
        a = 1 + 1
    %}

    return ();
}

func main() {
    assert_not_zero(0);
    assert_not_zero(1);
    let x = 500 * 5;
    assert_not_zero(x);
    tempvar y = -80;
    assert_not_zero(y);

    return ();
}
