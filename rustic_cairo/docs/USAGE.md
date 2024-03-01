<p align="center">
  <img src="./imgs/cairo_fuzzer.png" />
</p>

# Options:

```
Usage: cairo-fuzzer [OPTIONS]

Options:
      --cores <CORES>
          Set the number of threads to run [default: 1]
      --contract <CONTRACT>
          Set the path of the JSON artifact to load [default: ]
      --casm <CASM>
          Set the path of the JSON CASM artifact to load [default: ]
      --target-function <target_function>
          Set the function to fuzz [default: ]
      --statefull
          Keep the state of the fuzzer between runs
      --diff-fuzz
          diff fuzz between runs
      --corpus-dir <corpus_dir>
          Path to the inputs folder to load [default: ./corpus_dir]
      --crashes-dir <crashes_dir>
          Path to the crashes folder to load [default: ./crash_dir]
      --seed <SEED>
          Set a custom seed (only applicable for 1 core run) [default: 0]
      --config <CONFIG>
          Load config file
      --replay
          Replay the corpus folder
      --proptesting
          Property Testing
      --analyze
          Dump functions prototypes
  -h, --help
          Print help
```
## Fuzzing function of a contract:
```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --target-function "fuzzinglabs_starknet"
```

## Fuzzing function of a contract with statefull fuzzing:
```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --target-function "fuzzinglabs_starknet" --statefull
```

## Load old corpus:
```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --target-function "fuzzinglabs_starknet" --corpus_dir "./corpus_dir"
```

## Fuzzing using a config file:
Example of config file:
```json
{
    "cores":11,
    "seed": 4242,
    "contract": "./tests1.0/fuzzinglabs_fuzz.json",
    "casm": "./tests1.0/fuzzinglabs_fuzz.casm",
    "corpus_dir": "./corpus",
    "crashes_dir": "./crashes"
}
```

```sh
cargo run --release -- --config tests/config.json 
```

## Replay corpus folder:
```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm  --function "fuzzinglabs_starknet" --replay --crashes_dir "./crash_dir" 
```

## Fuzzing property testing:
Function should start with `Fuzz_`
```rust
    func Fuzz_symbolic_execution()
```

```sh
    cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm  --proptesting --iter 500000
```
