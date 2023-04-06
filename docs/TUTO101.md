# How to fuzz a Cairo/Starknet Smart Contract

We will take this Smart Contract as an example:
```rust
%builtins output
func Fuzz_symbolic_execution(
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
```

## Compile contract:
- Follow these [steps](https://www.cairo-lang.org/docs/quickstart.html) to setup cairo-lang in your environment
- Next, create the file `fuzzinglabs.cairo` that will contain the code above.
- run `cairo-compile fuzzinglabs.cairo --output fuzzinglabs.json`

## Analyze the code:
Looking at the code, we deduce that the function we want to fuzz is `Fuzz_symbolic_execution`, the goal is to find the good arguments to reach the `assert 0 = 2`.

## Running the fuzzer:
The simple command line to fuzz the function `Fuzz_symbolic_execution` of the `fuzzinglabs.cairo` contract is:

```sh
cargo run --release -- --cores 3 --contract tests/fuzzinglabs.json --function Fuzz_symbolic_execution
```

![fuzzer_running](fuzzer_running.png)

Understanding the output ` 1.00 uptime |     93000 fuzz cases |     92979.48 fcps |      5 coverage |      5 inputs |      0 crashes [     0 unique]`:
- 1.00 uptime: Number of seconds the fuzzer is running
- 93000 fuzz cases: Number of executions done
- 92979.48 fcps: Number of Fuzz Case Per Second
- 5 coverage: Number of instruction reached by the fuzzer
- 5 inputs: Number of interesting inputs that generate a new coverage
- 0 crashes [     0 unique]: Number of crashes and unique crashes

## Detecting the crash:
Once the fuzzer will find a unique crash you will have something like this:

![crash](crash.png)

You can see that the good input to reach the `assert 0 = 2` is `[102, 117, 122, 122, 105, 110, 103, 108, 97, 98, 115]`.
In ascii we get `[f,u,z,z,i,n,g,l,a,b,s]`.

So running the function `Fuzz_symbolic_execution` with `(102, 117, 122, 122, 105, 110, 103, 108, 97, 98, 115)` will lead to the assert.

## Optimize the fuzzing

You can optimize the fuzzing using the multiple option of Cairo-Fuzzer.
See [this documention](Usage.md) to get more information.