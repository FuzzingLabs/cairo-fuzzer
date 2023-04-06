%builtins output
func test_symbolic_execution(
    f: felt,
    u: felt,
    z: felt,
    z2: felt,
    i: felt,
    n: felt,
    g: felt,
    l: felt,
    a: felt,
    b: felt,
    s: felt,
) {
    if (f == 9992913) {
        if (u == 7423848) {
            if (z == 7214781287489724) {
                if (z2 == 757483838389399) {
                    if (i == 8247324828348) {
                        if (n == 7423848) {
                            if (g == 0) {
                                if (l == 2555) {
                                    if (a == 155) {
                                        if (b == 98 ) {
                                            if (s == 3) {
                                                assert 0 = 2;
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

func main{output_ptr: felt*}() {
    return ();
}
