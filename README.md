# Cairo-Fuzzer -- Cairo Smart Contract Fuzzer

Release version 1.2
Developped and maintained by [@FuzzingLabs](https://github.com/FuzzingLabs)

## Description:

Cairo-fuzzer is a tool designed for smart contract developers to test the security. It can be used as an independent tool or as a library.

## Features:

<p align="center">
	<img src="cairo-fuzzer.png"/>
</p>

- Run Starknet contract
- Replayer of fuzzing corpus
- Minimizer of fuzzing corpus
- Load old corpus
- Handle multiple arguments
- Workspace architecture
- Import dictionnary
- Use Cairo-fuzzer as a library


## Usage:
```
	cargo run --release -- --cores 10 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --function "Fuzz_symbolic_execution"

```

For more usage information, follow our [tutorial](docs/TUTO101.md)

## CMDLINE (--help):

```
Usage: cairo-fuzzer [OPTIONS]

Options:
      --cores <CORES>              Set the number of threads to run [default: 1]
      --contract <CONTRACT>        Set the path of the JSON artifact to load [default: ]
      --casm <CASM>                Set the path of the JSON CASM artifact to load [default: ]
      --function <FUNCTION>        Set the function to fuzz [default: ]
      --workspace <WORKSPACE>      Workspace of the fuzzer [default: fuzzer_workspace]
      --inputfolder <INPUTFOLDER>  Path to the inputs folder to load [default: ]
      --crashfolder <CRASHFOLDER>  Path to the crashes folder to load [default: ]
      --inputfile <INPUTFILE>      Path to the inputs file to load [default: ]
      --crashfile <CRASHFILE>      Path to the crashes file to load [default: ]
      --dict <DICT>                Path to the dictionnary file to load [default: ]
      --logs                       Enable fuzzer logs in file
      --seed <SEED>                Set a custom seed (only applicable for 1 core run)
      --run-time <RUN_TIME>        Number of seconds this fuzzing session will last
      --config <CONFIG>            Load config file
      --replay                     Replay the corpus folder
      --minimizer                  Minimize Corpora
      --proptesting                Property Testing
      --analyze                    Dump functions prototypes
      --iter <ITER>                Iteration Number [default: -1]
  -h, --help                       Print help
```

# F.A.Q

## How to find a Cairo/Starknet compilation artifact (json file)?

Cairo-Fuzzer supports starknet compilation artifact (json and casm files) generated after compilation using `starknet-compile` and `starknet-sierra-compile`.
Cairo-Fuzzer does not support Cairo2.0 and pure cairo contract.

## How to run the tests?

```
cargo test
```

# License

Cairo-Fuzzer is licensed and distributed under the AGPLv3 license. [Contact us](mailto:contact@fuzzinglabs.com) if you're looking for an exception to the terms.