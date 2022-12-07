### Basic run :

```sh
cargo run --release -- --cores 4 --contract tests/fuzzinglabs.json --function "test_symbolic_execution"
```

### --contract:
The path to the contract artifact we want to run.

### --function:
The name of the function we want to execute.

### --cores:
Spawn N number of threads

### --seed:
Set a custom seed.
