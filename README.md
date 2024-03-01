<p align="center">
  <img src="./docs/imgs/cairo_fuzzer.png" />
</p>

# Cairo-Fuzzer -- Cairo Smart Contract Fuzzer
Release version 2.0
Developped and maintained by [@FuzzingLabs](https://github.com/FuzzingLabs)
## Description:

Cairo-fuzzer is a tool designed for smart contract developers to test the security. It can be used as an independent tool or as a library.

## Features:

- Run Starknet contract
- Statefull and Stateless fuzzing
- Diffirential fuzzing (execute a call twice and verify that it generates the same trace) 
- Replayer of fuzzing corpus
- Load old corpus
- Handle multiple arguments
## CMDLINE (--help):

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
## Usage:
```
  cargo run --release -- --cores 11 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --target-function "Fuzz_symbolic_execution"
```
# F.A.Q

## How to find a Cairo/Starknet compilation artifact (json file)?

Cairo-Fuzzer supports starknet compilation artifact (json and casm files) generated after compilation using `starknet-compile` and `starknet-sierra-compile`.
Cairo-Fuzzer does not support Cairo2.0 and pure cairo contract.

# License

Cairo-Fuzzer is licensed and distributed under the AGPLv3 license. [Contact us](mailto:contact@fuzzinglabs.com) if you're looking for an exception to the terms.