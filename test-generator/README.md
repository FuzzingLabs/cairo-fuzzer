## Input file generator for the Cairo Fuzzer 

List the functions for which test case generation is possible : 

```bash
cargo run --bin test-generator ./examples/sierra/symbolic_execution_test.sierra 

Available functions:
        - symbolic::symbolic::symbolic_execution_test
```

Generate an inputfile for the cairo fuzzer : 

```bash
cargo run --bin test-generator ./examples/sierra/symbolic_execution_test.sierra  symbolic::symbolic::symbolic_execution_test > inputfile.json
```

It can now be used as an input file for the function we want to fuzz using the Cairo-fuzzer with the `--inputfile` parameter.