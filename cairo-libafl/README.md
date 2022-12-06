### Basic run :

```sh
cargo run -- --cores all --contract tests/fuzzinglabs.json --function "test_symbolic_execution"
```

### --contract:
The path to the contract artifact we want to run.

### --function:
The name of the function we want to execute.

### --cores:
Spawn a client in each of the provided cores. Broker runs in the 0th core. 'all' to select all available cores. 'none' to run a client without binding to any core. eg: '1,2-4,6' selects the cores 1,2,3,4,6.",
