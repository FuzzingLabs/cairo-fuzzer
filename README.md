# cairo-fuzzer

To run the Fuzzer:
```sh
cargo run -- --cores all --contract tests/fuzzinglabs.json --function "test_symbolic_execution"
```

### --cores
- all => use all the cores
- X => will use the X core
- X-Y => wi